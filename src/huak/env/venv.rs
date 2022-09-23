use std::{
    env::{self, consts::OS},
    path::{Path, PathBuf},
};

use crate::{
    errors::CliError, package::python::PythonPackage,
    utils::path::search_parents_for_filepath,
};

use super::python::PythonEnvironment;

const DEFUALT_SEARCH_STEPS: usize = 5;
pub(crate) const DEFAULT_VENV_NAME: &str = ".venv";
pub(crate) const BIN_NAME: &str = "bin";
pub(crate) const WINDOWS_BIN_NAME: &str = "Scripts";

/// A struct for Python venv.
#[derive(Clone)]
pub struct Venv {
    pub path: PathBuf,
}

impl Venv {
    /// Initialize a `Venv`.
    pub fn new(path: PathBuf) -> Venv {
        Venv { path }
    }

    /// Initialize a `Venv` by searching a directory for a venv. `from()` will search
    /// the parents directory for a configured number of recursive steps.
    // TODO: Improve the directory search (refactor manifest search into search utility).
    pub fn from(from: &Path) -> Result<Venv, anyhow::Error> {
        let names = vec![".venv", "venv"];

        // TODO: Redundancy.
        for name in &names {
            if let Ok(Some(path)) =
                search_parents_for_filepath(from, name, DEFUALT_SEARCH_STEPS)
            {
                return Ok(Venv::new(path));
            };
        }

        Err(anyhow::format_err!(
            "could not find venv from {}",
            from.display()
        ))
    }

    /// Get the name of the Venv (ex: ".venv").
    pub fn name(&self) -> Result<&str, anyhow::Error> {
        let name = crate::utils::path::parse_filename(self.path.as_path())?;

        Ok(name)
    }

    /// Create the venv at its path.
    pub fn create(&self) -> Result<(), CliError> {
        if self.path.exists() {
            return Ok(());
        }

        let from = match self.path.parent() {
            Some(p) => p,
            _ => {
                return Err(CliError::new(
                    anyhow::format_err!("invalid venv path"),
                    2,
                ))
            }
        };

        let name = self.name()?;
        let args = ["-m", "venv", name];

        crate::utils::command::run_command("python", &args, from)?;

        Ok(())
    }
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

impl PythonEnvironment for Venv {
    /// Get the path to the bin folder (called Scripts on Windows).
    fn bin_path(&self) -> PathBuf {
        match OS {
            "windows" => self.path.join(WINDOWS_BIN_NAME),
            _ => self.path.join(BIN_NAME),
        }
    }

    /// Get the path to the module passed from the venv.
    fn module_path(&self, module: &str) -> Result<PathBuf, anyhow::Error> {
        let bin_path = self.bin_path();
        let mut path = bin_path.join(module);

        if OS != "windows" {
            return Ok(path);
        }

        match path.set_extension("exe") {
            true => Ok(path),
            false => {
                Err(anyhow::format_err!("failed to create path for {module}"))
            }
        }
    }

    /// Run a module installed to the venv as an alias'd command from the current working dir.
    fn exec_module(
        &self,
        module: &str,
        args: &[&str],
        from: &Path,
    ) -> Result<(), CliError> {
        // Create the venv if it doesn't exist.
        // TODO: Fix this.
        self.create()?;

        let module_path = self.module_path(module)?;

        if !module_path.exists() {
            self.install_package(&PythonPackage::new(module.to_string()))?;
        }

        let module_path = crate::utils::path::to_string(module_path.as_path())?;

        crate::utils::command::run_command(module_path, args, from)?;

        Ok(())
    }

    /// Install a dependency to the venv.
    fn install_package(
        &self,
        dependency: &PythonPackage,
    ) -> Result<(), CliError> {
        let cwd = env::current_dir()?;
        let module_str = &dependency.string();
        let args = ["install", module_str];
        let module = "pip";

        self.exec_module(module, &args, cwd.as_path())?;

        Ok(())
    }

    /// Install a dependency from the venv.
    fn uninstall_package(&self, name: &str) -> Result<(), CliError> {
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
        let directory = tempdir().unwrap().into_path().to_path_buf();
        let first_venv = Venv::new(directory.join(".venv"));
        first_venv.create().unwrap();

        let second_venv = Venv::from(&directory).unwrap();

        assert!(second_venv.path.exists());
        assert!(second_venv.module_path("pip").unwrap().exists());
        assert_eq!(first_venv.path, second_venv.path);
    }
}
