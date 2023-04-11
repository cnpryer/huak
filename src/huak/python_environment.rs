use std::{
    cmp::Ordering,
    env::consts::OS,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use crate::{
    environment::env_path_values, fs, package::Package, sys, version::Version,
    Config, Error, HuakResult,
};

const DEFAULT_VENV_NAME: &str = ".venv";
const VENV_CONFIG_FILE_NAME: &str = "pyvenv.cfg";
const VIRTUAL_ENV_ENV_VAR: &str = "VIRTUAL_ENV";
const CONDA_ENV_ENV_VAR: &str = "CONDA_PREFIX";

/// The `PythonEnvironment` is a struct used to intereact with an environment
/// containing an installed Python `Interpreter` and `Package`s.
///
/// An example of a valid `PythonEnvironment` would be a Virtual environment.
/// See https://peps.python.org/pep-0405/
/// The structure of a `Venv` on a system depends on if it is Windows or not.
///
/// For Windows:
/// .venv (typically .venv or venv)
/// ├── Scripts
/// │   ├── python.exe
/// │   └── pip.exe
/// │── Lib
/// │   └── site-packages
/// └── pyvenv.cfg
///
/// Otherwise:
/// .venv (typically .venv or venv)
/// ├── bin
/// │   ├── python
/// │   └── pip
/// │── lib
/// │   └── python3.11
/// │      └── site-packages
/// └── pyvenv.cfg
///
/// Note that on Windows site-packages is under Lib but elsewhere it's under
/// lib/python{version-major.version-minor}. `pyvenv.cfg` is the `PythonEnvironment`'s
/// config file and contains information like the version of the Python
/// `Interpreter`, *potentially* the "home" path to the Python `Interpreter` that
/// generated the `PythonEnvironment`, etc.
///
/// ```
/// use huak::PythonEnvironment;
///
/// let venv = PythonEnvironment::new(".venv");
/// ```
pub struct PythonEnvironment {
    /// The absolute path to the `PythonEnvironment`'s root.
    root: PathBuf,
    /// The `PythonEnvironment`'s Python `Interpreter`.
    interpreter: Interpreter,
    /// The absolute path to the `PythonEnvironment`'s executables directory. This directory contains
    /// installed Python modules and the `Interpreter` the `Venv` uses. On Windows this
    /// is located at `PythonEnvironment.root\Scripts\`, otherwise it's located at
    /// `PythonEnvironment.root/bin/`
    executables_dir_path: PathBuf,
    /// The site-packages directory contains all of the `PythonEnvironment`'s installed Python packages.
    site_packages_path: PathBuf,
}

impl PythonEnvironment {
    /// Initialize a new `PythonEnvironment`.
    pub fn new<T: AsRef<Path>>(path: T) -> HuakResult<Self> {
        // Note that only virtual environments are supported at this time.
        if !path.as_ref().join(VENV_CONFIG_FILE_NAME).exists() {
            return Err(Error::Unimplemented(format!(
                "{} is not supported",
                path.as_ref().display()
            )));
        }

        let env = new_venv(path)?;

        Ok(env)
    }

    /// Get a reference to the path to the `PythonEnvironment`.
    pub fn root(&self) -> &Path {
        self.root.as_ref()
    }

    /// Get the name of the `PythonEnvironment`.
    pub fn name(&self) -> HuakResult<String> {
        fs::last_path_component(&self.root)
    }

    /// Get a reference to the Python `Interpreter`'s path that's used by the `PythonEnvironment`.
    pub fn python_path(&self) -> &PathBuf {
        self.interpreter.path()
    }

    /// Get a reference to the `PythonEnvironment`'s executables directory path.
    pub fn executables_dir_path(&self) -> &PathBuf {
        &self.executables_dir_path
    }

    /// Get a reference to the `PythonEnvironment`'s site-packages directory path.
    pub fn site_packages_dir_path(&self) -> &PathBuf {
        &self.site_packages_path
    }

    /// Install Python `Package`s to the `PythonEnvironment`.
    pub fn install_packages<T>(
        &self,
        packages: &[T],
        options: &InstallOptions,
        config: &Config,
    ) -> HuakResult<()>
    where
        T: Display,
    {
        let mut cmd = Command::new(self.python_path());
        cmd.args(["-m", "pip", "install"])
            .args(packages.iter().map(|item| item.to_string()));

        if let Some(v) = options.values.as_ref() {
            cmd.args(v.iter().map(|item| item.as_str()));
        }

        config.terminal().run_command(&mut cmd)
    }

    /// Uninstall Python `Package`s from the `PythonEnvironment`.
    pub fn uninstall_packages<T>(
        &self,
        packages: &[T],
        options: &InstallOptions,
        config: &Config,
    ) -> HuakResult<()>
    where
        T: Display,
    {
        let mut cmd = Command::new(self.python_path());
        cmd.args(["-m", "pip", "uninstall"])
            .args(packages.iter().map(|item| item.to_string()))
            .arg("-y");

        if let Some(v) = options.values.as_ref() {
            cmd.args(v.iter().map(|item| item.as_str()));
        }

        config.terminal().run_command(&mut cmd)
    }

    /// Update Python `Package`s installed in the `PythonEnvironment`.
    pub fn update_packages<T>(
        &self,
        packages: &[T],
        options: &InstallOptions,
        config: &Config,
    ) -> HuakResult<()>
    where
        T: Display,
    {
        let mut cmd = Command::new(self.python_path());
        cmd.args(["-m", "pip", "install", "--upgrade"])
            .args(packages.iter().map(|item| item.to_string()));

        if let Some(v) = options.values.as_ref() {
            cmd.args(v.iter().map(|item| item.as_str()));
        }

        config.terminal().run_command(&mut cmd)
    }

    /// Check if the `PythonEnvironment` has a module installed in the executables directory.
    pub fn contains_module(&self, module_name: &str) -> HuakResult<bool> {
        let dir = self.executables_dir_path();
        #[cfg(unix)]
        return Ok(dir.join(module_name).exists());
        #[cfg(windows)]
        {
            let mut path = dir.join(module_name);
            match path.set_extension("exe") {
                true => return Ok(path.exists()),
                false => Err(Error::InternalError(format!(
                    "failed to create path for {module_name}"
                ))),
            }
        }
    }

    #[allow(dead_code)]
    /// Check if the `PythonEnvironment` has a `Package` already installed.
    pub fn contains_package(&self, package: &Package) -> bool {
        self.site_packages_dir_path().join(package.name()).exists()
    }

    /// Get all of the `Package`s installed in the `PythonEnvironment`.
    pub fn installed_packages(&self) -> HuakResult<Vec<Package>> {
        let mut cmd = Command::new(self.python_path());
        cmd.args(["-m", "pip", "freeze"]);

        let output = cmd.output()?;
        let output = sys::parse_command_output(output)?;
        let mut packages = Vec::new();
        for line in output.split('\n') {
            if !line.is_empty() {
                packages.push(Package::from_str(line)?);
            }
        }

        Ok(packages)
    }

    /// Check if the `PythonEnvironment` is already activated.
    pub fn active(&self) -> bool {
        Some(&self.root)
            == active_virtual_env_path()
                .or(active_conda_env_path())
                .as_ref()
    }
}

/// Helper function for creating a new virtual environment as a `PythonEnvironment`.
fn new_venv<T: AsRef<Path>>(path: T) -> HuakResult<PythonEnvironment> {
    let root = path.as_ref();

    // Establishing paths differs between Windows and Unix systems.
    #[cfg(unix)]
    let executables_dir_path = root.join("bin");
    #[cfg(unix)]
    let python_path = executables_dir_path.join("python");
    #[cfg(windows)]
    let executables_dir_path = root.join("Scripts");
    #[cfg(windows)]
    let python_path = executables_dir_path.join("python.exe");

    let config = VenvConfig::new(root.join(VENV_CONFIG_FILE_NAME))?;
    let version = config.version;

    // On Unix systems the Venv's site-package directory depends on the Python version.
    // The path is root/lib/pythonX.X/site-packages.
    #[cfg(unix)]
    let site_packages_path = root
        .join("lib")
        .join(format!(
            "python{}.{}",
            version.release()[0],
            version.release()[1]
        ))
        .join("site-packages");
    #[cfg(windows)]
    let site_packages_path = root.join("Lib").join("site-packages");

    let interpreter = Interpreter {
        version,
        path: python_path,
    };

    let venv = PythonEnvironment {
        root: root.to_path_buf(),
        interpreter,
        executables_dir_path,
        site_packages_path,
    };

    Ok(venv)
}

#[derive(Clone)]
/// A struct used to configure Python `Package` installations.
pub struct InstallOptions {
    /// A values vector of install options typically used for passing on arguments.
    pub values: Option<Vec<String>>,
}

/// Python virtual environment configuration data (pyvenv.cfg).
///
/// See https://docs.python.org/3/library/venv.html.
struct VenvConfig {
    /// The `Version` of the virtual environment's Python `Interpreter`.
    version: Version,
}

impl VenvConfig {
    /// Initialize a new `VenvConfig` from the pvenv.cfg path.
    fn new<T: AsRef<Path>>(value: T) -> HuakResult<Self> {
        // Read the file and flatten the lines for parsing.
        let file = File::open(&value).unwrap_or_else(|_| {
            panic!("failed to open {}", value.as_ref().display())
        });
        let buff_reader = BufReader::new(file);
        let lines = buff_reader.lines().flatten().collect::<Vec<String>>();

        // Search for version = "X.X.X"
        let mut version = Version::from_str("0.0.0");
        lines.iter().for_each(|item| {
            let mut split = item.splitn(2, '=');
            let key = split.next().unwrap_or_default().trim();
            let val = split.next().unwrap_or_default().trim();
            if key == "version" {
                version = Version::from_str(val)
            }
        });

        let version = version.expect("Python version from pyvenv.cfg");
        let cfg = VenvConfig { version };

        Ok(cfg)
    }
}

/// A wrapper for a collection of `Interpreter`s.
///
/// Use `Interpreters` to access latest and exact Python `Interpreter`s by `Version.
/// You can also get an `Interpreter` by its path.
///
/// ```
/// use huak::Environment;
///
/// let interpreters = Environment::new().resolve_python_interpreters();
/// let python_path = interpreters.latest();
/// ```
pub struct Interpreters {
    interpreters: Vec<Interpreter>,
}

impl Interpreters {
    /// Initialize a new `Interpreters` wrapper.
    pub fn new<T>(interpreters: T) -> Interpreters
    where
        T: Iterator<Item = Interpreter>,
    {
        let interpreters = interpreters.collect::<Vec<_>>();

        Interpreters { interpreters }
    }

    /// Get a reference to the wrapped `Interpreter`s.
    pub fn interpreters(&self) -> &Vec<Interpreter> {
        &self.interpreters
    }

    #[allow(dead_code)]
    /// Get the latest Python `Interpreter` by `Version`.
    pub fn latest(&self) -> Option<&Interpreter> {
        self.interpreters.iter().max()
    }

    #[allow(dead_code)]
    /// Get a Python `Interpreter` by its `Version`.
    fn exact(&self, version: &Version) -> Option<&Interpreter> {
        self.interpreters
            .iter()
            .find(|interpreter| &interpreter.version == version)
    }
}

#[derive(Debug)]
/// The Python `Interpreter` is used to interact with installed Python `Interpreter`s.
///
/// `Interpreter` contains information like the `Interpreter`'s path, `Version`, etc.
///
/// ```
/// use huak::Interpreter;
///
/// let python = Interpreter::new("path/to/python");
/// ```
pub struct Interpreter {
    /// The `Version` of the Python `Interpreter`.
    version: Version,
    /// The absolute path to the Python `Interpreter`.
    path: PathBuf,
}

impl Interpreter {
    pub fn new<T: AsRef<Path>>(path: T, version: Version) -> Interpreter {
        let interpreter = Interpreter {
            version,
            path: path.as_ref().to_path_buf(),
        };

        interpreter
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn version(&self) -> &Version {
        &self.version
    }
}

impl Display for Interpreter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) {}", self.version(), self.path().display())
    }
}

impl PartialEq<Self> for Interpreter {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Interpreter {}

impl PartialOrd<Self> for Interpreter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Interpreter {
    fn cmp(&self, other: &Self) -> Ordering {
        match compare_interpreters(self, other) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Greater,
        }
    }
}

fn compare_interpreters(this: &Interpreter, other: &Interpreter) -> Ordering {
    if this.version != other.version {
        return this.version.cmp(&other.version);
    }

    Ordering::Equal
}

/// Get the VIRTUAL_ENV environment path if it exists.
pub fn active_virtual_env_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var(VIRTUAL_ENV_ENV_VAR) {
        return Some(PathBuf::from(path));
    }

    None
}

/// Get the CONDA_PREFIX environment path if it exists.
pub fn active_conda_env_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var(CONDA_ENV_ENV_VAR) {
        return Some(PathBuf::from(path));
    }

    None
}

pub fn venv_config_file_name() -> &'static str {
    VENV_CONFIG_FILE_NAME
}

pub fn default_venv_name() -> &'static str {
    DEFAULT_VENV_NAME
}

/// Get an `Iterator` over available Python `Interpreter` paths parsed from the `PATH`
/// environment variable (inspired by brettcannon/python-launcher).
pub fn python_paths() -> impl Iterator<Item = (Option<Version>, PathBuf)> {
    let paths =
        fs::flatten_directories(env_path_values().unwrap_or(Vec::new()));

    python_interpreters_in_paths(paths)
}

/// Get an `Iterator` over all found Python `Interpreter` paths with their `Version` if
/// one is found.
fn python_interpreters_in_paths(
    paths: impl IntoIterator<Item = PathBuf>,
) -> impl Iterator<Item = (Option<Version>, PathBuf)> {
    paths.into_iter().filter_map(|item| {
        item.file_name()
            .or(None)
            .and_then(|raw_file_name| raw_file_name.to_str().or(None))
            .and_then(|file_name| {
                if valid_python_interpreter_file_name(file_name) {
                    #[cfg(unix)]
                    {
                        if let Ok(version) =
                            version_from_python_interpreter_file_name(file_name)
                        {
                            Some((Some(version), item.clone()))
                        } else {
                            None
                        }
                    }
                    #[cfg(windows)]
                    Some((
                        version_from_python_interpreter_file_name(file_name)
                            .ok(),
                        item.clone(),
                    ))
                } else {
                    None
                }
            })
    })
}

#[cfg(unix)]
/// A function for checking if a Python `Interpreter`'s file name is valid.
fn valid_python_interpreter_file_name(file_name: &str) -> bool {
    if file_name == "python" {
        return true;
    }

    if !file_name.starts_with("python") {
        return false;
    }

    file_name.len() >= "python3.0".len()
        && file_name["python".len()..].parse::<f32>().is_ok()
}

#[cfg(windows)]
/// A function for checking if a Python `Interpreter`'s file name is valid.
fn valid_python_interpreter_file_name(file_name: &str) -> bool {
    if file_name == "python.exe" || file_name == "python" {
        return true;
    }

    if !file_name.starts_with("python") {
        return false;
    }

    let name = file_name.strip_suffix(".exe").unwrap_or(file_name);

    name.len() > "python".len() && name["python".len()..].parse::<f32>().is_ok()
}

/// Parse the `Version` from a Python `Interpreter`'s file name.
///
/// On Windows we strip the .exe extension and attempt the parse.
/// On Unix we just attempt the parse immediately.
fn version_from_python_interpreter_file_name(
    file_name: &str,
) -> HuakResult<Version> {
    match OS {
        "windows" => Version::from_str(
            &file_name.strip_suffix(".exe").unwrap_or(file_name)
                ["python".len()..],
        ),
        _ => Version::from_str(&file_name["python".len()..]),
    }
    .map_err(|_| {
        Error::InternalError(format!("could not version from {file_name}"))
    })
}

pub fn parse_python_version_from_command<T: AsRef<Path>>(
    path: T,
) -> HuakResult<Option<Version>> {
    let mut cmd = Command::new(path.as_ref());
    cmd.args([
        "-c",
        "import sys;v=sys.version_info;print(v.major,v.minor,v.micro)",
    ]);
    let output = sys::parse_command_output(cmd.output()?)?
        .replace(' ', ".")
        .replace(['\r', '\n'], "");
    let version = Version::from_str(&output).ok();

    Ok(version)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::TerminalOptions;

    use super::*;

    #[test]
    fn python_environment_executables_dir_name() {
        let dir = tempdir().unwrap();
        let config = Config {
            workspace_root: dir.path().to_path_buf(),
            cwd: dir.path().to_path_buf(),
            terminal_options: TerminalOptions {
                verbosity: sys::Verbosity::Quiet,
            },
        };
        let ws = config.workspace();
        let venv = ws.resolve_python_environment().unwrap();

        assert!(venv.executables_dir_path.exists());
        #[cfg(unix)]
        assert!(venv.executables_dir_path.join("python").exists());
        #[cfg(windows)]
        assert!(venv.executables_dir_path().join("python.exe").exists());
    }

    #[test]
    fn find_python() {
        let path = python_paths().next().unwrap().1;

        assert!(path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn python_search() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("python3.11"), "").unwrap();
        let path_vals = vec![dir.path().to_str().unwrap().to_string()];
        std::env::set_var("PATH", path_vals.join(":"));
        let mut interpreter_paths = python_paths();

        assert_eq!(
            interpreter_paths.next().unwrap().1,
            dir.path().join("python3.11")
        );
    }

    #[cfg(windows)]
    #[test]
    fn python_search() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("python.exe"), "").unwrap();
        let path_vals = vec![dir.path().to_str().unwrap().to_string()];
        std::env::set_var("PATH", path_vals.join(":"));
        let mut interpreter_paths = python_paths();

        assert_eq!(
            interpreter_paths.next().unwrap().1,
            dir.path().join("python.exe")
        );
    }
}
