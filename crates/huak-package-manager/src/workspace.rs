use crate::package::Package;
use crate::settings::SettingsDb;
use crate::{
    environment::Environment,
    fs,
    metadata::LocalMetadata,
    python_environment::{default_venv_name, venv_config_file_name},
    Config, Error, HuakResult, PythonEnvironment,
};
use huak_toolchain::{Channel, LocalToolchain, LocalToolchainResolver};
use std::{path::PathBuf, process::Command};
use toml_edit::{Item, Value};

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

    /// Get the current `Package`. The current `Package` is one found by its metadata file nearest based
    /// on the `Workspace`'s `Config` data.
    pub fn current_package(&self) -> HuakResult<Package> {
        // Currently only pyproject.toml `LocalMetadata` file is supported.
        let metadata = self.current_local_metadata()?;

        let package = Package::from(metadata.metadata().clone());

        Ok(package)
    }

    /// Get the current `LocalMetadata` based on the `Config` data.
    pub fn current_local_metadata(&self) -> HuakResult<LocalMetadata> {
        let package_root = find_package_root(&self.config.cwd, &self.root)?;

        // Currently only pyproject.toml is supported.
        let path = package_root.join("pyproject.toml");
        let metadata = LocalMetadata::new(path)?;

        Ok(metadata)
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
        let env = PythonEnvironment::new(path)?;

        Ok(env)
    }

    /// Create a `PythonEnvironment` for the `Workspace`.
    fn new_python_environment(&self) -> HuakResult<PythonEnvironment> {
        // Get a snapshot of the environment.
        let env = self.environment();

        // Get the first Python `Interpreter` path found from the `PATH`
        // environment variable.
        let Some(python_path) = env.python_paths().next() else {
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

/// Search for a Python `Package` root.
/// 1. Walk from the `from` dir upwards, searching for dir containing the `LocalMetadata` file.
/// 2. Stop after searching the `stop_after` dir.
pub fn find_package_root<T: Into<PathBuf>>(from: T, stop_after: T) -> HuakResult<PathBuf> {
    let from = from.into();
    let stop_after = stop_after.into();

    if !from.is_dir() || !stop_after.is_dir() {
        return Err(Error::InternalError(
            "`from` and `stop_after` must be directoreis".to_string(),
        ));
    }

    // Currently only pyproject.toml is supported
    let file_path = match fs::find_root_file_bottom_up("pyproject.toml", from, stop_after) {
        Ok(it) => it.ok_or(Error::MetadataFileNotFound)?,
        Err(_) => return Err(Error::MetadataFileNotFound),
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
    if let Some(channel) = channel.as_ref() {
        let resolver = LocalToolchainResolver::new();
        return resolver.from_dir(channel, toolchains);
    }

    // Use workspace project metadata and return if a toolchain is listed.
    if let Ok(metadata) = workspace.current_local_metadata() {
        if let Some(table) = metadata.metadata().tool().and_then(|it| it.get("huak")) {
            if let Some(path) = table
                .get("toolchain")
                .map(std::string::ToString::to_string)
                .map(PathBuf::from)
            {
                if path.exists() {
                    return Some(LocalToolchain::new(path));
                }
            };
        };
    };

    // Attempt to retrieve the toolchain for the current workspace scope.
    if let Some(table) = SettingsDb::try_from(settings)
        .ok()
        .as_ref()
        .and_then(|db| db.doc().as_table()["scopes"].as_table())
    {
        if let Some(Item::Value(Value::String(s))) = table.get(&format!("{}", config.cwd.display()))
        {
            return Some(LocalToolchain::new(PathBuf::from(s.to_string())));
        }
    }

    None
}
