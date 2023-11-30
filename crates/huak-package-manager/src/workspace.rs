use crate::package::Package;
use crate::{
    environment::Environment,
    fs,
    manifest::LocalManifest,
    python_environment::{default_venv_name, venv_config_file_name},
    Config, Error, HuakResult, PythonEnvironment,
};
use huak_toolchain::{Channel, LocalToolchain, LocalToolchainResolver, SettingsDb};
use huak_workspace::{resolve_first, PathMarker};
use std::str::FromStr;
use std::{path::PathBuf, process::Command};
use toml_edit::Item;

/// The `Workspace` is a struct for resolving things like the current `Package`
/// or the current `PythonEnvironment`. It can also provide a snapshot of the `Environment`,
/// a more general struct containing information like environment variables, Python
/// `Interpreters` found, etc.
///
/// ```
/// use huak_package_manager::Workspace;
///
/// let workspace = Workspace::new(".");
/// let env = workspace.environment();
/// ```
pub struct Workspace {
    /// The established `Workspace` root path.
    root: PathBuf,
    /// The `Config` associated with the `Workspace`.
    config: Config,
}

impl Workspace {
    pub fn new<T: Into<PathBuf>>(path: T, config: &Config) -> Self {
        Workspace {
            root: path.into(),
            config: config.clone(),
        }
    }

    /// Get a reference to the path to the `Workspace` root.
    #[must_use]
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Get an `Environment` associated with the `Workspace`.
    #[must_use]
    pub fn environment(&self) -> Environment {
        Environment::new()
    }

    /// Get the current `Package`. The current `Package` is one found by its manifest file nearest based
    /// on the `Workspace`'s `Config` data.
    pub fn current_package(&self) -> HuakResult<Package> {
        // Currently only pyproject.toml `LocalManifest` file is supported.
        let manifest = self.current_local_manifest()?;

        let package = Package::try_from_manifest(&manifest)?;

        Ok(package)
    }

    /// Get the current `LocalManifest` based on the `Config` data.
    pub fn current_local_manifest(&self) -> HuakResult<LocalManifest> {
        // The current manifest file is the first found in a search.
        let ws = resolve_first(&self.config.cwd, PathMarker::file("pyproject.toml"));

        // Currently only pyproject.toml is supported.
        let path = ws.root().join("pyproject.toml");

        if path.exists() {
            LocalManifest::new(path)
        } else {
            Err(Error::ManifestFileFound)
        }
    }

    /// Resolve a `PythonEnvironment` pulling the current or creating one if none is found.
    pub fn resolve_python_environment(&self) -> HuakResult<PythonEnvironment> {
        // NOTE: Currently only virtual environments are supported. We search for them, stopping
        // at the configured workspace root. If none is found we create a new one at the
        // workspace root.
        let env = match self.current_python_environment() {
            Ok(it) => it,
            Err(Error::PythonEnvironmentNotFound) => self.new_python_environment()?,
            Err(e) => return Err(e),
        };

        Ok(env)
    }

    /// Get the current `PythonEnvironment`. The current `PythonEnvironment` is one
    /// found by its configuration file or `Interpreter` nearest baseed on `Config` data.
    pub fn current_python_environment(&self) -> HuakResult<PythonEnvironment> {
        let path = find_venv_root(&self.config.cwd, &self.root)?;
        let py_env = PythonEnvironment::new(path)?;

        Ok(py_env)
    }

    /// Create a `PythonEnvironment` for the `Workspace`.
    fn new_python_environment(&self) -> HuakResult<PythonEnvironment> {
        // Get a snapshot of the environment.
        let env = self.environment();
        // Include toolchain installations when resolving for a Python interpreter to use.
        // If a toolchain cannot be resolved then the first Python path found from the
        // environment is used.
        let Some(python_path) = self
            .resolve_local_toolchain(None)
            .ok()
            .and_then(|tc| {
                // TODO(cnpryer): Proxy better + Refactor
                // We use the venv Python.
                PythonEnvironment::new(tc.root().join(".venv"))
                    .ok()
                    .map(|venv| venv.python_path().to_owned())
            })
            .or_else(|| env.python_paths().next().map(PathBuf::from))
        else {
            return Err(Error::PythonNotFound);
        };

        // Set the name and path of the `PythonEnvironment. Note that we currently only
        // support virtual environments.
        let name = default_venv_name();
        let path = self.root.join(name);

        // Create the `PythonEnvironment`. This uses the `venv` module distributed with Python.
        // Note that this will fail on systems with minimal Python distributions.
        let args = ["-m", "venv", name];
        let mut cmd = Command::new(python_path);
        cmd.args(args).current_dir(&self.root);
        self.config.terminal().run_command(&mut cmd)?;

        let python_env = PythonEnvironment::new(path)?;

        Ok(python_env)
    }

    /// Get the current toolchain. The current toolchain is found by:
    /// 1. `HUAK_TOOLCHAIN` environment variable
    /// 2. [tool.huak.toolchain] pyproject.toml configuration
    /// 3. ~/.huak/settings.toml configuration
    pub fn resolve_local_toolchain(&self, channel: Option<&Channel>) -> HuakResult<LocalToolchain> {
        let Some(it) = resolve_local_toolchain(self, channel) else {
            return Err(Error::ToolchainNotFound);
        };

        Ok(it)
    }
}

/// A struct used to configure options for `Workspace`s.
pub struct WorkspaceOptions {
    /// Inidcate the `Workspace` should use git.
    pub uses_git: bool,
    /// Trailing argument values.
    pub values: Option<Vec<String>>,
}

/// Search for a Python virtual environment.
/// 1. If `VIRTUAL_ENV` exists then a venv is active; use it.
/// 2. Walk from the `from` dir upwards, searching for dir containing the pyvenv.cfg file.
/// 3. Stop after searching the `stop_after` dir.
pub fn find_venv_root<T: Into<PathBuf>>(from: T, stop_after: T) -> HuakResult<PathBuf> {
    let from = from.into();
    let stop_after = stop_after.into();

    if let Ok(path) = std::env::var("VIRTUAL_ENV") {
        return Ok(PathBuf::from(path));
    }

    if !from.is_dir() || !stop_after.is_dir() {
        return Err(Error::InternalError(
            "`from` and `stop_after` must be directories".to_string(),
        ));
    }

    let file_path = match fs::find_root_file_bottom_up(venv_config_file_name(), from, stop_after) {
        Ok(it) => it.ok_or(Error::PythonEnvironmentNotFound)?,
        Err(_) => return Err(Error::PythonEnvironmentNotFound),
    };

    // The root of the venv is always the parent dir to the pyvenv.cfg file.
    let root = file_path
        .parent()
        .ok_or(Error::InternalError(
            "failed to establish parent directory".to_string(),
        ))?
        .to_path_buf();

    Ok(root)
}

// TODO(cnpryer): Channel must be compatible with HUAK_TOOLCHAIN if found
fn resolve_local_toolchain(
    workspace: &Workspace,
    channel: Option<&Channel>,
) -> Option<LocalToolchain> {
    let config = &workspace.config;

    let Some(home) = config.home.as_ref() else {
        return None;
    };

    let toolchains = home.join("toolchains");
    let settings = toolchains.join("settings.toml");

    // Use an environment variable if it's active.
    if let Ok(path) = std::env::var("HUAK_TOOLCHAIN").map(PathBuf::from) {
        if path.exists() {
            return Some(LocalToolchain::new(path));
        }
    }

    // If a channel is provided then search for it from huak's toolchain directory.
    if let Some(channel) = channel {
        let resolver = LocalToolchainResolver::new();
        return resolver.from_dir(channel, toolchains);
    }

    // Use workspace project manifest and return if a toolchain is listed. TODO(cnpryer): May not be channel
    if let Some(manifest_channel) = workspace
        .current_local_manifest()
        .map(|it| {
            it.manifest_data()
                .tool_table()
                .and_then(|tool| tool.get("huak"))
                .and_then(Item::as_table)
                .and_then(|table| table.get("toolchain"))
                .and_then(Item::as_str) // TODO(cnpryer): Support non-string toolchain values
                .and_then(|s| Channel::from_str(s).ok())
        })
        .unwrap_or_default()
        .as_ref()
    {
        return resolve_local_toolchain(workspace, Some(manifest_channel));
    };

    // Attempt to retrieve the toolchain for the current workspace scope by resolving for
    // the first matching path from cwd.
    if let Some(db) = SettingsDb::try_from(settings).ok().as_ref() {
        for p in config.cwd.ancestors() {
            // TODO(cnpryer): Should report better
            if let Ok(Some((_, value))) = db.get_scope_entry(p) {
                return Some(LocalToolchain::new(PathBuf::from(value.to_string())));
            }
        }
    }

    None
}
