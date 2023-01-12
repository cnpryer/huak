use std::{
    env::{self, consts::OS},
    path::{Path, PathBuf},
};

#[allow(clippy::useless_attribute)]
#[allow(unused_imports)]
use crate::{
    errors::{HuakError, HuakResult},
    package::python::PythonPackage,
    utils::{
        path::search_parents_for_filepath,
        shell::{get_shell_name, get_shell_path, get_shell_source_command},
    },
};

const DEFAULT_SEARCH_STEPS: usize = 5;
pub(crate) const DEFAULT_VENV_NAME: &str = ".venv";
pub(crate) const DEFAULT_PYTHON_ALIAS: &str = "python";
pub(crate) const BIN_NAME: &str = "bin";
pub(crate) const WINDOWS_BIN_NAME: &str = "Scripts";
pub(crate) const HUAK_VENV_ENV_VAR: &str = "HUAK_VENV_ACTIVE";

/// A struct for Python venv.
#[derive(Clone)]
pub struct Venv {
    pub path: PathBuf,
}

impl Default for Venv {
    fn default() -> Venv {
        let cwd = match env::current_dir() {
            Err(_) => Path::new(".").to_path_buf(),
            Ok(p) => p,
        };

        Venv {
            path: cwd.join(DEFAULT_VENV_NAME),
        }
    }
}

impl Venv {
    /// Initialize a `Venv`.
    pub fn new(path: PathBuf) -> Venv {
        Venv { path }
    }

    /// Initialize a `Venv` by searching a directory for a venv. `from()` will search
    /// the parents directory for a configured number of recursive steps.
    // TODO: Improve the directory search (refactor manifest search into search utility).
    pub fn from_path(from: &Path) -> HuakResult<Venv> {
        let names = vec![".venv", "venv"];

        // TODO: Redundancy.
        for name in &names {
            if let Ok(Some(path)) =
                search_parents_for_filepath(from, name, DEFAULT_SEARCH_STEPS)
            {
                return Ok(Venv::new(path));
            };
        }

        Err(HuakError::VenvNotFound)
    }

    /// Get the name of the Venv (ex: ".venv").
    pub fn name(&self) -> HuakResult<&str> {
        let name = crate::utils::path::parse_filename(self.path.as_path())?;

        Ok(name)
    }

    /// Activates the virtual environment in the current shell
    pub fn activate(&self) -> HuakResult<()> {
        // Check if venv is already activated
        if env::var(HUAK_VENV_ENV_VAR).is_ok() {
            return Err(HuakError::VenvActive);
        }

        let script = self.get_activation_script()?;
        if !script.exists() {
            return Err(HuakError::VenvNotFound);
        }
        let source_command = get_shell_source_command()?;
        let activation_command =
            format!("{} {}", source_command, script.display());

        env::set_var(HUAK_VENV_ENV_VAR, "1");
        self.spawn_pseudo_terminal(&activation_command)?;

        Ok(())
    }

    /// Spawn a pseudo-terminal with current shell and source activation script
    #[cfg(unix)]
    fn spawn_pseudo_terminal(
        &self,
        activation_command: &str,
    ) -> HuakResult<()> {
        let shell_path = get_shell_path()?;
        let mut new_shell = expectrl::spawn(&shell_path)?;
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
    fn spawn_pseudo_terminal(
        &self,
        activation_command: &str,
    ) -> HuakResult<()> {
        let shell_path = get_shell_path()?;
        let mut sh = expectrl::spawn(shell_path)?;
        let stdin = expectrl::stream::stdin::Stdin::open()?;

        sh.send_line(&activation_command)?;

        sh.interact(stdin, std::io::stdout()).spawn()?;

        let stdin = expectrl::stream::stdin::Stdin::open()?;
        stdin.close()?;
        Ok(())
    }

    /// Gets path to the activation script
    /// (e.g. `.venv/bin/activate`)
    ///
    /// Takes current shell into account.
    /// Returns errors if it fails to get correct env vars.
    fn get_activation_script(&self) -> HuakResult<PathBuf> {
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
            .bin_path()
            .join(Path::new(&("activate".to_owned() + suffix)));

        Ok(path)
    }
    /// Create the venv at its configured path if it doesn't already
    /// exist.
    pub fn create(&self) -> HuakResult<()> {
        if self.path.exists() {
            return Ok(());
        }

        let from = match self.path.parent() {
            Some(p) => p,
            _ => {
                return Err(HuakError::ConfigurationError(
                    "Invalid venv path, no parent directory.".into(),
                ))
            }
        };

        let name = self.name()?;
        let args = ["-m", "venv", name];

        println!("Creating venv {}", self.path.display());

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
                    HuakError::PythonNotFound => {
                        DEFAULT_PYTHON_ALIAS.to_string()
                    }
                    _ => return Err(e),
                }
            }
        };

        crate::utils::command::run_command(&py, &args, from)?;

        Ok(())
    }

    /// Get the python binary from the venv.
    pub fn python_binary(&self) -> HuakResult<String> {
        let path = crate::env::system::find_python_binary_path(Some(
            self.path.to_path_buf(),
        ))?;

        Ok(path)
    }

    /// Get the path to the bin folder (called Scripts on Windows).
    pub fn bin_path(&self) -> PathBuf {
        match OS {
            "windows" => self.path.join(WINDOWS_BIN_NAME),
            _ => self.path.join(BIN_NAME),
        }
    }

    /// Get the path to the module passed from the venv.
    pub fn module_path(&self, module: &str) -> HuakResult<PathBuf> {
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

    /// Run a module installed to the venv as an aliased command from the current working dir.
    pub fn exec_module(
        &self,
        module: &str,
        args: &[&str],
        from: &Path,
    ) -> HuakResult<()> {
        // Create the venv if it doesn't exist.
        // TODO: Fix this.
        self.create()?;

        let module_path = self.module_path(module)?;
        let package = match PythonPackage::from(module) {
            Ok(it) => it,
            // TODO: Don't do this post-decouple.
            Err(_) => {
                return Err(HuakError::PyPackageInitError(module.to_string()))
            }
        };

        if !module_path.exists() {
            self.install_package(&package)?;
        }

        let module_path = crate::utils::path::to_string(module_path.as_path())?;

        crate::utils::command::run_command(module_path, args, from)?;

        Ok(())
    }

    /// Run a command in the context of the venv.
    pub fn exec_command(&self, command: &str) -> HuakResult<()> {
        // Create the venv if it doesn't exist.
        // TODO: Fix this.
        self.create()?;

        let source_command = get_shell_source_command()?;
        let script = self.get_activation_script()?;
        let activation_command =
            format!("{} {}", source_command, script.display());

        let shell_path = get_shell_path()?;
        let cwd = env::current_dir()?;
        crate::utils::command::run_command(
            &shell_path,
            &["-c", &format!("{} && {}", activation_command, command)],
            cwd.as_path(),
        )?;

        Ok(())
    }

    /// Install a Python package to the venv.
    pub fn install_package(&self, package: &PythonPackage) -> HuakResult<()> {
        let cwd = env::current_dir()?;
        let module_str = &package.string();
        let args = ["install", module_str];
        let module = "pip";

        self.exec_module(module, &args, cwd.as_path())?;

        Ok(())
    }

    /// Uninstall a dependency from the venv.
    pub fn uninstall_package(&self, name: &str) -> HuakResult<()> {
        let cwd = env::current_dir()?;
        let module = "pip";
        let args = ["uninstall", name, "-y"];

        self.exec_module(module, &args, cwd.as_path())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn default() {
        let venv = Venv::default();

        assert!(venv.path.ends_with(DEFAULT_VENV_NAME));
    }

    #[test]
    fn from() {
        let directory = tempdir().unwrap().into_path();
        let first_venv = Venv::new(directory.join(".venv"));
        first_venv.create().unwrap();

        let second_venv = Venv::from_path(&directory).unwrap();

        assert!(second_venv.path.exists());
        assert!(second_venv.module_path("pip").unwrap().exists());
        assert_eq!(first_venv.path, second_venv.path);
    }
}
