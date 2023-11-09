use crate::{Config, Error, HuakResult, Verbosity};
use huak_home::huak_home_dir;
use huak_python_manager::{
    resolve_release, PythonManager, Release, ReleaseArchitecture, ReleaseBuildConfiguration,
    ReleaseKind, ReleaseOption, ReleaseOptions, ReleaseOs, RequestedVersion, Strategy, Version,
};
use huak_toolchain::{Channel, DescriptorParts, LocalTool, LocalToolchain, SettingsDb};
use sha2::{Digest, Sha256};
use std::{
    env::consts::OS,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};
use termcolor::Color;

/// Resolve the target toolchain if a user provides one, otherwise get the current toolchain
/// for the current workspace. If no toolchain is found then emit "error: no toolchain found".
/// Add the user-provided tool to the toolchain. If the tool is
/// already installed to the toolchain, and a version is provided that's different from the
/// installed tool, then replace the installed tool with the desired version.
pub fn add_tool(tool: &LocalTool, channel: Option<&Channel>, config: &Config) -> HuakResult<()> {
    // Resolve a toolchain if a channel is provided. Otherwise resolve the curerent.
    let toolchain = config.workspace().resolve_local_toolchain(channel)?;

    let tool = toolchain.tool(&tool.name);
    let args = [
        "-m".to_string(),
        "pip".to_string(),
        "install".to_string(),
        tool.to_string(),
    ];
    let py = toolchain.tool("python");
    let py_bin = py_bin(toolchain.downloads().join("python"));

    let mut terminal = config.terminal();

    let mut cmd = Command::new(py.path);
    let cmd = cmd.args(args).current_dir(&config.cwd);

    terminal.print_custom(
        "Updating",
        format!("adding {} to {}", &tool.name, toolchain.name()),
        Color::Green,
        true,
    )?;

    // terminal.set_verbosity(Verbosity::Quiet);

    // terminal.set_verbosity(Verbosity::Quiet);
    terminal.run_command(cmd)?;

    toolchain.register_tool_from_path(py_bin.join(&tool.name), &tool.name, false)?;

    terminal.set_verbosity(Verbosity::Normal);

    terminal.print_custom(
        "Success",
        format!("{} was added to '{}'", &tool.name, toolchain.name()),
        Color::Green,
        true,
    )
}

/// Resolve the target toolchain if a user provides one, otherwise get the current toolchain
/// for the current workspace. If no toolchain is found then emit "error: no toolchain found".
///
/// Display the toolchain's information:
///
/// Toolchain: <toolchain name>
/// Path: <toolchain path>
/// Channel: <toolchain channel>
/// Tools:
///   python (<veserion>)
///   ruff (<version>)
///   mypy (<version>)
///   pytest (<version>)
pub fn toolchain_info(channel: Option<&Channel>, config: &Config) -> HuakResult<()> {
    let toolchain = config.workspace().resolve_local_toolchain(channel)?;

    config
        .terminal()
        .print_without_status(toolchain.info(), Color::White)
}

/// Resolve and install a toolchain to some target directory using a channel.
pub fn install_toolchain(
    channel: Option<Channel>,
    target: Option<PathBuf>,
    config: &Config,
) -> HuakResult<()> {
    // If a toolchain cannot be resolved with a channel or the current config data then the default
    // will be installed if it doesn't already exist.
    let ws = config.workspace();

    if let Ok(toolchain) = ws.resolve_local_toolchain(channel.as_ref()) {
        return Err(Error::LocalToolchainExists(toolchain.root().clone()));
    }

    // If no target path is provided we always install to Huak's toolchain directory
    let Some(parent) = target.or(huak_home_dir().map(|it| it.join("toolchains"))) else {
        return Err(Error::InternalError(
            "target path is invalid or missing".to_string(),
        ));
    };

    let channel = channel.unwrap_or_default();
    let channel_string = channel.to_string();
    let path = parent.join(&channel_string);

    if path.exists() {
        return Err(Error::LocalToolchainExists(path));
    }

    if let Err(e) = install(path.clone(), channel, config) {
        teardown(parent.join(&channel_string), config)?;
        Err(e)
    } else {
        Ok(())
    }
}

#[allow(clippy::too_many_lines)]
fn install(path: PathBuf, channel: Channel, config: &Config) -> HuakResult<()> {
    let mut toolchain = LocalToolchain::new(path);

    toolchain.set_channel(channel);

    let name = toolchain.name();

    // We'll emit messages to the terminal for each tool installed.
    let mut terminal = config.terminal();

    // Get the tool 'python' from the toolchain.
    let py = toolchain.tool("python");

    // If 'python' is already installed we don't install it.
    if py.exists() {
        terminal.print_warning(format!(
            "Toolchain already exists at {}",
            toolchain.bin().display()
        ))?;

        return Ok(());
    }

    let root = toolchain.root();
    for p in [root.join("bin"), root.join("downloads"), root.join("venvs")] {
        std::fs::create_dir_all(p)?;
    }

    // Determine what Python release data to use for the install.
    let Some(release) = python_release_from_channel(toolchain.channel()) else {
        return Err(Error::PythonReleaseNotFound(
            toolchain.channel().to_string(),
        ));
    };
    let release_string = release.to_string();

    let msg = if matches!(toolchain.channel(), Channel::Default) {
        format!("toolchain '{}' ({})", toolchain.name(), release)
    } else {
        format!("toolchain '{}'", toolchain.name())
    };

    terminal.print_custom("Installing", msg, Color::Green, true)?;

    // Begin preparing to install 'python'.
    terminal.print_custom(
        "Preparing",
        format!("release validation for {release}"),
        Color::Green,
        true,
    )?;

    // Set up a manager to help with the Python installation process.
    let py_manager = PythonManager::new();

    // Download the release for installation.
    let buff = py_manager.download(release)?;
    let release_bytes = buff.as_slice();

    // If the checksum we generate from the downloaded data does not match the checksum we get
    // with the toolchain tool then we don't install it.
    let checksum = generate_checksum(release_bytes);
    if !checksum.eq_ignore_ascii_case(release.checksum) {
        return Err(Error::InvalidChecksum(release.to_string()));
    }

    terminal.print_custom("Success", format!("verified {release}"), Color::Green, true)?;
    terminal.print_custom(
        "Downloading",
        format!("release from {}", release.url),
        Color::Green,
        true,
    )?;

    // Extract the downloaded release to the toolchain's downloads directory.
    let downloads_dir = toolchain.downloads();
    terminal.print_custom(
        "Extracting",
        format!("{} to {}", release_string, downloads_dir.display()),
        Color::Green,
        true,
    )?;

    // Unpack the encoded archive bytes into the toolchains downloads dir.
    py_manager.unpack(release_bytes, &downloads_dir, true)?;

    // Get the path to the unpacked contents.
    let py_bin = py_bin(toolchain.downloads().join("python"));
    let py_path = maybe_exe(py_bin.join(format!(
        "python{}.{}",
        release.version.major, release.version.minor
    )));

    terminal.print_custom("Installing", release_string, Color::Green, true)?;

    // Use the installed python
    let py = LocalTool::new(py_path);

    if py.exists() {
        terminal.print_custom(
            "Preparing",
            "toolchain's virtual environment",
            Color::Green,
            true,
        )?;
    } else {
        return Err(Error::InternalError(format!(
            "'{}' could not be found",
            py.name
        )));
    }

    // Python is used from a dedicated virtual environment.
    let from = toolchain.root().join("venvs");
    std::fs::create_dir_all(&from)?;

    let mut cmd: Command = Command::new(py.path);
    cmd.current_dir(&from).args(["-m", "venv", &name]);
    terminal.run_command(&mut cmd)?;

    let venv = from.join(name);
    let path = venv.join(python_bin_name()).join("python");

    terminal.print_custom(
        "Success",
        format!("prepared virtual environment for '{}'", toolchain.name()),
        Color::Green,
        true,
    )?;

    terminal.print_custom(
        "Updating",
        "toolchain bin with python".to_string(),
        Color::Green,
        true,
    )?;

    // Try to link the tool in the bin directory as a proxy. If that fails copy the tool entirely.
    for it in [
        "python".to_string(),
        "python3".to_string(),
        format!(
            "python{}.{}",
            &release.version.major, &release.version.minor
        ),
    ]
    .as_ref()
    {
        if toolchain.register_tool_from_path(&path, it, false).is_err() {
            if let Err(e) = toolchain.register_tool_from_path(&path, it, true) {
                return Err(Error::ToolchainError(e));
            }
        }
    }

    terminal.print_custom(
        "Success",
        format!("installed python ({})", toolchain.bin().display()),
        Color::Green,
        true,
    )?;

    let py = toolchain.tool("python");

    for name in default_python_tools() {
        terminal.set_verbosity(Verbosity::Quiet);

        let mut cmd: Command = Command::new(&py.path);
        cmd.current_dir(&config.cwd)
            .args(["-m", "pip", "install", name]);

        terminal.run_command(&mut cmd)?;

        // If the python is a symlink then use the bin its linked to. Otherwise use the venv path.
        let path = py_bin.join(name);

        // Register the installed python module as a proxy.
        toolchain.register_tool_from_path(&path, name, false)?;

        terminal.set_verbosity(Verbosity::Normal);
        terminal.print_custom(
            "Success",
            format!("installed {name} ({})", toolchain.bin().display()),
            Color::Green,
            true,
        )?;
    }

    terminal.print_custom(
        "Finished",
        format!(
            "installed '{}' ({})",
            toolchain.name(),
            toolchain.root().display()
        ),
        Color::Green,
        true,
    )
}

/// Resolve available toolchains and display their names as a list. Display the following with
///
/// Current toolchain: <toolchain name>
///
/// Installed toolchains:
/// 1: <toolchain name>
/// 2: <toolchain name>
/// 3: <toolchain name>
pub fn list_toolchains(config: &Config) -> HuakResult<()> {
    let mut terminal = config.terminal();

    if let Ok(current_toolchain) = config.workspace().resolve_local_toolchain(None) {
        terminal.print_custom(
            "Current:",
            current_toolchain.root().display(),
            Color::Cyan,
            true,
        )?;
    }

    if let Some(toolchains) = resolve_installed_toolchains(config) {
        if !toolchains.is_empty() {
            terminal.print_custom("Installed", "", Color::Green, true)?;

            for (i, toolchain) in toolchains.iter().enumerate() {
                config.terminal().print_custom(
                    format!("{:>5})", i + 1),
                    format!("{:<16}", toolchain.name()),
                    Color::Green,
                    true,
                )?;
            }
        }
    }

    Ok(())
}

/// Resolve the target toolchain but don't perform and installs if none can be found. If a toolchain
/// can be resolved (located) then remove the tool. If the tool is not installed to the toolchain then
/// exit silently.
pub fn remove_tool(tool: &LocalTool, channel: Option<&Channel>, config: &Config) -> HuakResult<()> {
    if tool.name == "python" {
        unimplemented!()
    }

    // Resolve a toolchain if a channel is provided. Otherwise resolve the curerent.
    let toolchain = config.workspace().resolve_local_toolchain(channel)?;

    let tool = toolchain.tool(&tool.name);
    let args = ["-m", "pip", "uninstall", &tool.name, "-y"];
    let py = toolchain.tool("python");

    let mut terminal = config.terminal();

    let mut cmd = Command::new(py.path);
    let cmd = cmd.args(args).current_dir(&config.cwd);

    terminal.print_custom(
        "Updating",
        format!("removing {} from '{}'", &tool.name, toolchain.name()),
        Color::Green,
        true,
    )?;

    terminal.set_verbosity(Verbosity::Quiet);

    terminal.run_command(cmd)?;

    remove_path_with_scope(&tool.path, toolchain.root())?;

    terminal.set_verbosity(Verbosity::Normal);

    terminal.print_custom(
        "Success",
        format!("{} was uninstalled", &tool.name),
        Color::Green,
        true,
    )
}

/// Resolve the target toolchain but don't perform and installs if none can be found. If a toolchain
/// can be resolved (located) then run the tool. If the tool is not installed to the toolchain then
/// emit "error: a problem occurred running a tool: {tool} is not installed"
pub fn run_tool(
    tool: &LocalTool,
    channel: Option<&Channel>,
    trailing: Option<Vec<String>>,
    config: &Config,
) -> HuakResult<()> {
    let ws = config.workspace();

    let toolchain = ws.resolve_local_toolchain(channel)?;

    run(
        toolchain.tool(&tool.name),
        trailing.unwrap_or_default().as_slice(),
        config,
    )
}

fn run(tool: LocalTool, args: &[String], config: &Config) -> HuakResult<()> {
    let mut terminal = config.terminal();
    let mut cmd: Command = Command::new(tool.path);
    cmd.args(args).current_dir(&config.cwd);
    terminal.run_command(&mut cmd)
}

/// Resolve the target toolchain but don't perform and installs if none can be found. If a toolchain
/// can be resolved (located) then uninstall it.
pub fn uninstall_toolchain(channel: Option<&Channel>, config: &Config) -> HuakResult<()> {
    let ws = config.workspace();
    let toolchain = ws.resolve_local_toolchain(channel)?;

    let mut terminal = config.terminal();

    terminal.print_custom(
        "Updating",
        format!(
            "uninstalling '{}' ({})",
            toolchain.name(),
            toolchain.root().display()
        ),
        Color::Green,
        true,
    )?;

    // TODO: Outside home
    remove_path_with_scope(toolchain.root(), config.home.as_ref().expect("huak home"))?;

    if let Some(parent) = toolchain.root().parent() {
        let settings = parent.join("settings.toml");

        if let Ok(db) = SettingsDb::try_from(&settings).as_mut() {
            db.remove_toolchain(toolchain.root());
            db.save(&settings)?;
        }
    }

    terminal.print_custom("Success", "toolchain uninstalled", Color::Green, true)
}

/// Resolve the target toolchain but don't perform and installs if none can be found. If a toolchain
/// can be resolved (located) then attempt to update its tools according to its channel. If the channel
/// is version-defined without a patch number then install the latest released Python for that channel.
/// Update the rest of the tools in the toolchain.
pub fn update_toolchain(
    tool: Option<LocalTool>,
    channel: Option<&Channel>,
    config: &Config,
) -> HuakResult<()> {
    // Resolve a toolchain if a channel is provided. Otherwise resolve the curerent.
    let toolchain = config.workspace().resolve_local_toolchain(channel)?;

    let mut terminal = config.terminal();
    let tools = if let Some(it) = tool {
        vec![it]
    } else {
        toolchain
            .tools()
            .into_iter()
            .filter(|it| it.name != "python")
            .chain([LocalTool {
                name: "pip".to_string(),
                path: toolchain.bin().join("pip"),
            }])
            .collect()
    };

    let py = toolchain.tool("python");

    let args = ["-m", "pip", "install", "--upgrade"];
    for tool in tools {
        let mut cmd = Command::new(&py.path);

        terminal.print_custom("Updating", &tool.name, Color::Green, true)?;
        terminal.set_verbosity(Verbosity::Quiet);

        cmd.args(args.iter().chain([&tool.name.as_str()]))
            .current_dir(&config.cwd);

        terminal.run_command(&mut cmd)?;

        terminal.set_verbosity(Verbosity::Normal);
    }

    terminal.print_custom("Success", "finished updating", Color::Green, true)
}

// Resolve the target toolchain if a user provides one, otherwise get the current toolchain
// for the current workspace. If none can be found then install and use the default toolchain.
// Update the settings.toml with the scope that should *use* the resolved toolchain.
pub fn use_toolchain(channel: &Channel, config: &Config) -> HuakResult<()> {
    let ws = config.workspace();

    let Some(home) = config.home.as_ref() else {
        return Err(Error::HuakHomeNotFound);
    };

    let toolchain = ws.resolve_local_toolchain(Some(channel))?;
    let settings = home.join("toolchains").join("settings.toml");
    let mut db = SettingsDb::try_from(&settings).unwrap_or_default();

    db.insert_scope(ws.root(), toolchain.root());

    Ok(db.save(settings)?)
}

fn resolve_installed_toolchains(config: &Config) -> Option<Vec<LocalToolchain>> {
    let Some(home) = config.home.clone() else {
        return None;
    };

    let Ok(toolchains) = std::fs::read_dir(home.join("toolchains")) else {
        return None;
    };

    let mut chains = Vec::new();

    for entry in toolchains.flatten() {
        let p = entry.path();

        if p.is_dir() && p.parent().map_or(false, |it| it == home.join("toolchains")) {
            chains.push(LocalToolchain::new(p));
        }
    }

    Some(chains)
}

fn generate_checksum(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);

    hex::encode(hasher.finalize())
}

fn python_release_from_channel(channel: &Channel) -> Option<Release<'static>> {
    let options = match channel {
        Channel::Default => ReleaseOptions::default(), // TODO(cnpryer): Is there ever a case where channel default doesn't yield python default?
        Channel::Version(version) => release_options_from_version(*version),
        Channel::Descriptor(descriptor) => release_options_from_descriptor(descriptor),
    };

    resolve_release(&Strategy::Selection(options))
}

fn release_options_from_descriptor(descriptor: &DescriptorParts) -> ReleaseOptions {
    let desc = descriptor.clone();
    let kind = desc.kind.unwrap_or(ReleaseKind::default().to_string());
    let os = desc.os.unwrap_or(ReleaseOs::default().to_string());
    let architecture = desc
        .architecture
        .unwrap_or(ReleaseArchitecture::default().to_string());
    let build_configuration = desc
        .build_configuration
        .unwrap_or(ReleaseBuildConfiguration::default().to_string());

    ReleaseOptions {
        kind: ReleaseOption::from_str(&kind).ok(),
        version: desc.version.map(|it| {
            ReleaseOption::Version(RequestedVersion {
                major: it.major,
                minor: it.minor,
                patch: it.patch,
            })
        }),
        os: ReleaseOption::from_str(&os).ok(),
        architecture: ReleaseOption::from_str(&architecture).ok(),
        build_configuration: ReleaseOption::from_str(&build_configuration).ok(),
    }
}

fn release_options_from_version(version: Version) -> ReleaseOptions {
    ReleaseOptions {
        kind: Some(ReleaseOption::Kind(ReleaseKind::default())),
        version: Some(ReleaseOption::Version(RequestedVersion {
            major: version.major,
            minor: version.minor,
            patch: version.patch,
        })),
        os: Some(ReleaseOption::Os(ReleaseOs::default())),
        architecture: Some(ReleaseOption::Architecture(ReleaseArchitecture::default())),
        build_configuration: Some(ReleaseOption::BuildConfiguration(
            ReleaseBuildConfiguration::default(),
        )),
    }
}

fn python_bin_name() -> &'static str {
    match OS {
        "windows" => "Scripts",
        _ => "bin",
    }
}

fn py_bin<T: AsRef<Path>>(root: T) -> PathBuf {
    root.as_ref().join("install").join(python_bin_name())
}

// TODO: Refactor
fn maybe_exe(path: PathBuf) -> PathBuf {
    if OS == "windows" && path.extension().map_or(false, |it| it == "exe") {
        path.with_extension("exe")
    } else {
        path
    }
}

fn default_python_tools() -> [&'static str; 3] {
    ["ruff", "pytest", "mypy"]
}

fn teardown<T: AsRef<Path>>(path: T, config: &Config) -> HuakResult<()> {
    let path = path.as_ref();

    if let Some(home) = config.home.as_ref() {
        remove_path_with_scope(path, home)
    } else {
        Ok(())
    }
}

fn remove_path_with_scope<T, R>(path: T, root: R) -> HuakResult<()>
where
    T: AsRef<Path>,
    R: AsRef<Path>,
{
    let path = path.as_ref();
    let root = root.as_ref();

    let mut stack = vec![path.to_path_buf()];

    while let Some(mut p) = stack.pop() {
        p.pop();

        if p == root {
            if p.is_dir() {
                std::fs::remove_dir_all(path)?;
                return Ok(());
            } else if p.is_file() {
                std::fs::remove_file(path)?;
                return Ok(());
            }
        } else {
            stack.push(p);
        }
    }

    Ok(())
}
