use std::{
    env::{self, consts::OS},
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use crate::{
    errors::{HuakError, HuakResult},
    package::{dist_info::DistInfo, PythonPackage},
    utils::{
        self,
        path::search_directories_for_file,
        shell::{get_shell_name, get_shell_path},
    },
};

use super::system::find_python_binary_path;

const DEFAULT_SEARCH_STEPS: usize = 5;
const DEFAULT_VENV_NAME: &str = ".venv";
const DEFAULT_PYTHON_ALIAS: &str = "python";
const UNIX_BIN_DIRECTORY_NAME: &str = "bin";
const WINDOWS_SCRIPT_DIRECTORY_NAME: &str = "Scripts";
const VIRTUAL_ENVIRONMENT_CONFIG_NAME: &str = "pyvenv.cfg";
const VENV_ENV_VAR: &str = "VIRTUAL_ENV";
const CONDA_ENV_VAR: &str = "CONDA_PREFIX";
const CONDA_DEFAULT_ENV_VAR: &str = "CONDA_DEFAULT_ENV";

// A Python environment to Huak is an environment that
//   - Python packages can be installed to
//   - Can be used as a Python workflow context
//     - Run commands against
//     - Create and destroy
//     - ...
//   - Is PEP compliant
//       lib
//         └── pythonX.XX
//           └── site-packages
//             ├── some_pkg
//             └── some_pkg-X.X.X.dist-info
//
// The following types of Python environments are expected to be compatible
// with Hauk:
//   - Virtual Environments
//   - Global site-packages directory
//   - (Eventually) __pypackages__ directory
pub trait PythonEnvironment {
    /// Creates the Python environment on the system.
    fn create(&self) -> HuakResult<()>;
    /// Get the name of the Python environment.
    fn name(&self) -> HuakResult<&str>;
    /// Get the full path to the environment directory (root).
    fn path(&self) -> &PathBuf;
    /// Get the full path the the executable binary for the Python interpreter the
    /// Python environment uses.
    fn interpreter_path(&self) -> &PathBuf;
    /// Get the full path the the executable binary for the Python interpreter that
    /// was used to create the environment (same as interpreter_path if it's the
    /// original system installation).
    fn base_interpreter_path(&self) -> &PathBuf;
    /// Get the full path to site-packages directory.
    fn site_packages_path(&self) -> &PathBuf;
    /// Get the name of the bin directory. On windows this is called Scripts.
    fn bin_name(&self) -> &str;
    /// Get the path to the bin directory. On wintows this ends with Scripts.
    fn bin_path(&self) -> PathBuf;
    /// Get the path to a Python module (executable or psuedo executable Python program)
    /// installed to the bin or Scripts directory.
    fn module_path(&self, module: &str) -> HuakResult<PathBuf>;
    /// Get the distribution information for each installed package.
    fn package_dist_info(
        &self,
        package: &PythonPackage,
    ) -> HuakResult<Option<DistInfo>>;
    /// Ensure that a Python package is installed to the environment.
    fn package_is_installed(&self, package: &PythonPackage) -> bool;
}

pub trait Activatable {
    /// Activate a Python environment.
    fn activate(&self) -> HuakResult<()>;
    /// Get the Python environment's activation script path.
    fn get_activation_script_path(&self) -> HuakResult<PathBuf>;
}

pub struct Venv {
    data: EnvironmentData,
}

impl Venv {
    pub fn new(path: &Path) -> Venv {
        let mut data = EnvironmentData::new();
        data.path = path.to_path_buf();

        Venv { data }
    }

    /// Initialize a `Venv` by searching a directory for a venv. `from_directory()` will search
    /// the parents directory for a configured number of recursive steps. A virtual environment
    /// is identified by its configuration file (pyvenv.cfg).
    pub fn from_directory(path: &Path) -> HuakResult<Venv> {
        let config_path = search_directories_for_file(
            path,
            VIRTUAL_ENVIRONMENT_CONFIG_NAME,
            search_max_steps(),
        )?
        .ok_or(HuakError::PyVenvNotFoundError)?;
        let data = environment_data_from_venv_config_path(&config_path)?;

        Ok(Venv { data })
    }

    fn data(&self) -> &EnvironmentData {
        &self.data
    }

    /// Check if the Venv is a valid Python environment that can be used.
    pub fn validate(&self) -> HuakResult<()> {
        // TODO: Make this more robust.
        if self.path().exists() {
            return Ok(());
        }

        Err(HuakError::PyVenvNotFoundError)
    }
}

impl Default for Venv {
    fn default() -> Venv {
        let cwd = env::current_dir().unwrap_or_default();
        Venv::new(&cwd.join(DEFAULT_VENV_NAME))
    }
}

impl PythonEnvironment for Venv {
    fn create(&self) -> HuakResult<()> {
        if self.path().exists() {
            return Ok(());
        }

        let from = match self.path().parent() {
            Some(p) => p,
            _ => {
                return Err(HuakError::HuakConfigurationError(
                    "Invalid venv path, no parent directory.".into(),
                ))
            }
        };

        let name = self.name()?;
        let args = ["-m", "venv", name];

        println!("Creating venv {}", self.path().display());

        // Create venv using system binary found from PATH variable.
        // TODO: Refactor implementation for searching for binary since this is redundant for
        //       systems with the Python bin path added to the PATH. Those systems should
        //       have an alias available anyway. We want the create method to attempt to
        //       locate a Python binary on the system if it isn't added to PATH.
        let py = match crate::env::system::find_python_binary_path(None) {
            Ok(it) => it,
            Err(e) => {
                match e {
                    // See TODO comment above. Windows PATH variable search is
                    // incomplete, so this will attempt the alias if it's on the
                    // PATH.
                    HuakError::PythonNotFoundError => {
                        PathBuf::from(DEFAULT_PYTHON_ALIAS.to_string())
                    }
                    _ => return Err(e),
                }
            }
        };

        crate::utils::command::run_command(
            utils::path::to_string(&py)?,
            &args,
            from,
        )?;

        Ok(())
    }

    fn name(&self) -> HuakResult<&str> {
        let name = crate::utils::path::parse_filename(self.path().as_path())?;

        Ok(name)
    }

    fn path(&self) -> &PathBuf {
        &self.data.path
    }

    fn bin_path(&self) -> PathBuf {
        self.path().join(bin_name())
    }

    fn bin_name(&self) -> &str {
        bin_name()
    }

    fn module_path(&self, module: &str) -> HuakResult<PathBuf> {
        let bin_path = self.bin_path();
        let mut path = bin_path.join(module);

        if OS != "windows" {
            return Ok(path);
        }

        match path.set_extension("exe") {
            true => Ok(path),
            false => Err(HuakError::InternalError(format!(
                "failed to create path for {module}"
            ))),
        }
    }

    fn interpreter_path(&self) -> &PathBuf {
        self.data().interpreter_path()
    }

    fn base_interpreter_path(&self) -> &PathBuf {
        self.data().base_interpreter_path()
    }

    fn site_packages_path(&self) -> &PathBuf {
        &self.data().site_packages_path
    }

    fn package_dist_info(
        &self,
        package: &PythonPackage,
    ) -> HuakResult<Option<DistInfo>> {
        DistInfo::from_package(package, &self.data.site_packages_path)
    }

    // NOTE: Hacky use of package_dist_info (TODO?)
    fn package_is_installed(&self, package: &PythonPackage) -> bool {
        matches!(self.package_dist_info(package).ok(), Some(it) if it.is_some())
    }
}

impl Activatable for Venv {
    /// Activates the virtual environment in the current shell
    fn activate(&self) -> HuakResult<()> {
        // Check if venv is already activated
        if env::var(VENV_ENV_VAR).is_ok() | env::var(CONDA_ENV_VAR).is_ok() {
            return Ok(());
        }

        let script = self.get_activation_script_path()?;
        if !script.exists() {
            return Err(HuakError::PyVenvNotFoundError);
        }
        let activation_command = match OS {
            "windows" => utils::path::to_string(&script)?.to_string(),
            _ => format!("source {}", script.display()),
        };

        spawn_pseudo_terminal(&activation_command)?;

        Ok(())
    }

    /// Gets path to the activation script
    /// (e.g. `.venv/bin/activate`)
    ///
    /// Takes current shell into account.
    /// Returns errors if it fails to get correct env vars.
    fn get_activation_script_path(&self) -> HuakResult<PathBuf> {
        let shell_name = get_shell_name()?;

        let suffix = match shell_name.as_str() {
            "fish" => ".fish",
            "csh" | "tcsh" => ".csh",
            "powershell" | "pwsh" => ".ps1",
            "cmd" => ".bat",
            "nu" => ".nu",
            _ => "",
        };

        let path = self
            .path()
            .join(bin_name())
            .join(Path::new(&("activate".to_owned() + suffix)));

        Ok(path)
    }
}

// Helper function to create a venv from a path. If it already exists, initialize
// and return the Venv. If it doesn't then create a .venv at the path given.
pub fn create_venv(dirpath: &Path) -> HuakResult<Venv> {
    let venv = match Venv::from_directory(dirpath) {
        Ok(it) => it,
        Err(HuakError::PyVenvNotFoundError) => {
            Venv::new(&dirpath.join(DEFAULT_VENV_NAME))
        }
        Err(e) => return Err(e),
    };

    // Attempt to create the venv. If it already exists nothing will happen.
    venv.create()?;

    Ok(venv)
}

fn environment_data_from_venv_config_path(
    path: &PathBuf,
) -> HuakResult<EnvironmentData> {
    let root = path.parent().ok_or(HuakError::PythonNotFoundError)?;
    let config = VirtualConfig::from_config_path(path)?;
    let interpreter = Interpreter {
        path: root.join(bin_name()).join("python"),
        version: config.version.to_string(),
    };
    let base_interpreter = Interpreter {
        path: config
            .executable
            .unwrap_or(find_python_binary_path(None).unwrap_or_default()), // TODO: Search on windows isn't great yet.
        version: config.version,
    };
    #[cfg(windows)]
    let site_packages_path =
        path.join("lib").join("python").join("site-packages");
    #[cfg(unix)]
    let site_packages_path = path
        .join("lib")
        .join(format!(
            "python{}",
            interpreter
                .version
                .splitn(2, '.')
                .collect::<Vec<&str>>()
                .join(".")
        ))
        .join("site-packages");
    let bin_path = path.join(bin_name());

    Ok(EnvironmentData {
        path: root.to_path_buf(),
        interpreter,
        base_interpreter,
        site_packages_path,
        bin_path,
    })
}

/// Spawn a pseudo-terminal with current shell and source activation script
#[cfg(unix)]
fn spawn_pseudo_terminal(activation_command: &str) -> HuakResult<()> {
    let shell_path = get_shell_path()?;
    let mut new_shell = expectrl::spawn(shell_path)?;
    let mut stdin = expectrl::stream::stdin::Stdin::open()?;
    new_shell.send_line(activation_command)?;
    if let Some((cols, rows)) = terminal_size::terminal_size() {
        new_shell
            .set_window_size(cols.0, rows.0)
            .map_err(|e| HuakError::InternalError(e.to_string()))?;
    }
    new_shell.interact(&mut stdin, std::io::stdout()).spawn()?;
    stdin.close()?;
    Ok(())
}

/// Spawn a pseudo-terminal with current shell and source activation script
#[cfg(windows)]
fn spawn_pseudo_terminal(activation_command: &str) -> HuakResult<()> {
    let shell_path = get_shell_path()?;
    let mut sh = expectrl::spawn(shell_path)?;
    let stdin = expectrl::stream::stdin::Stdin::open()?;

    sh.send_line(&activation_command)?;

    sh.interact(stdin, std::io::stdout()).spawn()?;

    let stdin = expectrl::stream::stdin::Stdin::open()?;
    stdin.close()?;
    Ok(())
}

fn search_max_steps() -> usize {
    env::var("HUAK_VENV_SEARCH_MAX_STEPS")
        .ok()
        .map_or(DEFAULT_SEARCH_STEPS, |s| {
            s.parse().ok().unwrap_or(DEFAULT_SEARCH_STEPS)
        })
}

fn bin_name() -> &'static str {
    match OS {
        "windows" => WINDOWS_SCRIPT_DIRECTORY_NAME,
        _ => UNIX_BIN_DIRECTORY_NAME,
    }
}

pub fn venv_env_var() -> &'static str {
    VENV_ENV_VAR
}

pub fn conda_env_var() -> &'static str {
    CONDA_ENV_VAR
}

pub fn conda_default_env_var() -> &'static str {
    CONDA_DEFAULT_ENV_VAR
}

pub fn conda_env_is_base() -> bool {
    CONDA_DEFAULT_ENV_VAR == "base"
}

#[derive(Default)]
pub struct EnvironmentData {
    pub path: PathBuf,
    interpreter: Interpreter,
    base_interpreter: Interpreter,
    pub site_packages_path: PathBuf,
    pub bin_path: PathBuf,
}

impl EnvironmentData {
    pub fn new() -> EnvironmentData {
        EnvironmentData::default()
    }

    pub fn interpreter_path(&self) -> &PathBuf {
        &self.interpreter.path
    }

    pub fn interpreter_version(&self) -> &String {
        &self.interpreter.version
    }

    pub fn base_interpreter_path(&self) -> &PathBuf {
        &self.base_interpreter.path
    }

    pub fn base_interpreter_version(&self) -> &String {
        &self.base_interpreter.version
    }
}

/// A public trait modeled after the Python Virtual Environment (Venv).
pub trait Virtual {
    /// Execute a command to activate the Python environment from within a terminal.
    fn activate(&self);
    /// Get the full file path to the script used to activate the environment.
    fn activation_script_path(&self) -> &PathBuf;
    /// Get the data from the Python environment's configuration file.
    fn config_data(&self) -> &VirtualConfig;
    /// Get the full filepath to the configuration file used by the virtual Python environment
    /// if one exists.
    fn config_filepath(&self) -> Option<&PathBuf>;
    /// Boolean value indicating if the global site-packages are available to the virtual
    /// environment. Being an isolated environment means that this value is False.
    fn is_isolated(&self) -> &bool;
}

/// Configuration data for Python virtual environments modeled after the pyvenv.cfg.
#[derive(Clone, Debug)]
pub struct VirtualConfig {
    pub home: PathBuf,
    pub include_system_site_packages: bool,
    // TODO: Proper version string parsing
    pub version: String,
    pub executable: Option<PathBuf>,
    pub command: Option<String>,
}

impl VirtualConfig {
    /// Construct the `VirtualConfig` from the Python virtual environment pyvenv.cfg file.
    pub fn from_config_path(path: &PathBuf) -> HuakResult<VirtualConfig> {
        let file = File::open(path)?;
        let buff_reader = BufReader::new(file);
        let lines: Vec<String> = buff_reader.lines().flatten().collect();
        let (
            mut home,
            mut include_system_site_packages,
            mut version,
            mut executable,
            mut command,
        ) = (PathBuf::new(), false, "".to_string(), None, None);
        for line in lines {
            let mut vals = line.splitn(2, " = ");
            let name = vals.next().unwrap_or_default();
            let value = vals.next().unwrap_or_default();
            if name == "home" {
                home = PathBuf::from(value);
            } else if name == "include-system-site-packages" {
                include_system_site_packages =
                    value.parse().ok().unwrap_or_default();
            } else if name == "version" {
                version = value.to_string();
            } else if name == "executable" {
                executable = Some(PathBuf::from(value));
            } else if name == "command" {
                command = Some(value.to_string());
            }
        }

        Ok(VirtualConfig {
            home,
            include_system_site_packages,
            version,
            executable,
            command,
        })
    }
}

#[derive(Default)]
struct Interpreter {
    path: PathBuf,
    version: String,
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn default() {
        let venv = Venv::default();

        assert!(venv.path().ends_with(DEFAULT_VENV_NAME));
    }

    #[test]
    fn from_directory() {
        let directory = tempdir().unwrap().into_path();
        let first_venv = Venv::new(&directory.join(".venv"));
        first_venv.create().unwrap();
        let second_venv = Venv::from_directory(&directory).unwrap();

        assert!(second_venv.path().exists());
        assert!(second_venv.module_path("pip").unwrap().exists());
        assert_eq!(first_venv.path(), second_venv.path());
    }

    #[test]
    fn test_bin_name() {
        #[cfg(windows)]
        assert_eq!(bin_name(), "Scripts");
        #[cfg(unix)]
        assert_eq!(bin_name(), "bin");
    }
}
