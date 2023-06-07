use crate::package::Package;
use crate::{
    environment::Environment,
    fs,
    metadata::LocalMetadata,
    python_environment::{default_venv_name, venv_config_file_name},
    Config, Error, HuakResult, PythonEnvironment,
};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// The `Workspace` is a struct for resolving things like the current `Package`
/// or the current `PythonEnvironment`. It can also provide a snapshot of the `Environment`,
/// a more general struct containing information like environment variables, Python
/// `Interpreters` found, etc.
///
/// ```
/// use huak_ops::Workspace;
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
    pub fn new<T: AsRef<Path>>(path: T, config: &Config) -> Self {
        let workspace = Workspace {
            root: path.as_ref().to_path_buf(),
            config: config.clone(),
        };

        workspace
    }

    /// Get a reference to the path to the `Workspace` root.
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Get an `Environment` associated with the `Workspace`.
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
            Err(Error::PythonEnvironmentNotFound) => {
                self.new_python_environment()?
            }
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
        let python_path = match env.python_paths().next() {
            Some(it) => it,
            None => return Err(Error::PythonNotFound),
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
}

/// A struct used to configure options for `Workspace`s.
pub struct WorkspaceOptions {
    /// Inidcate the `Workspace` should use git.
    pub uses_git: bool,
}

/// Search for a Python virtual environment.
/// 1. If VIRTUAL_ENV exists then a venv is active; use it.
/// 2. Walk from the `from` dir upwards, searching for dir containing the pyvenv.cfg file.
/// 3. Stop after searching the `stop_after` dir.
pub fn find_venv_root<T: AsRef<Path>>(
    from: T,
    stop_after: T,
) -> HuakResult<PathBuf> {
    if let Ok(path) = std::env::var("VIRTUAL_ENV") {
        return Ok(PathBuf::from(path));
    }

    if !from.as_ref().is_dir() || !stop_after.as_ref().is_dir() {
        return Err(Error::InternalError(
            "`from` and `stop_after` must be directories".to_string(),
        ));
    }

    let file_path = match fs::find_root_file_bottom_up(
        venv_config_file_name(),
        from,
        stop_after,
    ) {
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
pub fn find_package_root<T: AsRef<Path>>(
    from: T,
    stop_after: T,
) -> HuakResult<PathBuf> {
    if !from.as_ref().is_dir() || !stop_after.as_ref().is_dir() {
        return Err(Error::InternalError(
            "`from` and `stop_after` must be directoreis".to_string(),
        ));
    }

    // Currently only pyproject.toml is supported
    let file_path = match fs::find_root_file_bottom_up(
        "pyproject.toml",
        from,
        stop_after,
    ) {
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
