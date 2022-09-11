use std::{
    env::{self, consts::OS},
    path::{Path, PathBuf},
};

use crate::{config::requirements::PythonPackage, errors::CliError};

use super::python::PythonEnvironment;

const DEFAULT_VENV_NAME: &str = ".venv";
const BIN_NAME: &str = "bin";
const WINDOWS_BIN_NAME: &str = "Scripts";

/// A struct for Python venv.
#[derive(Clone)]
pub struct Venv {
    pub path: PathBuf,
}

impl Venv {
    pub fn new(path: PathBuf) -> Venv {
        Venv { path }
    }

    /// Initialize a `Venv` by searching a directory for a venv. This function will only check for
    /// .venv and venv at the root.
    // TODO: Improve the directory search.
    pub fn find(from: &Path) -> Result<Option<Venv>, anyhow::Error> {
        let paths = vec![from.join(".venv"), from.join("venv")];

        for path in &paths {
            if path.exists() {
                return Ok(Some(Venv {
                    path: path.to_path_buf(),
                }));
            }
        }

        Err(anyhow::format_err!("no venv found"))
    }

    /// Get the name of the Venv (ex: ".venv").
    pub fn name(&self) -> Result<&str, anyhow::Error> {
        let name = crate::utils::path::parse_filename(self.path.as_path())?;

        Ok(name)
    }

    /// Create the venv at its path.
    pub fn create(&self) -> Result<(), anyhow::Error> {
        if self.path.exists() {
            return Ok(());
        }

        let from = match self.path.parent() {
            Some(p) => p,
            _ => return Err(anyhow::format_err!("invalid venv path")),
        };

        let name = self.name()?;
        let args = ["-m", "venv", name];

        // Create venv using system's Python alias.
        if let Err(e) =
            crate::utils::command::run_command("python", &args, from)
        {
            return Err(e.error.unwrap_or_else(|| {
                anyhow::format_err!("failed to create venv")
            }));
        };

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

    /// Run a module installed to the venv as an alias'd command from the current working dir.
    fn exec_module(
        &self,
        module: &str,
        args: &[&str],
        from: &Path,
    ) -> Result<(), CliError> {
        let module_path = self.bin_path().join(module);
        let module_path = crate::utils::path::as_string(module_path.as_path())?;

        crate::utils::command::run_command(module_path, args, from)?;

        Ok(())
    }

    /// Install a dependency to the venv.
    fn install_package(
        &self,
        dependency: &PythonPackage,
    ) -> Result<(), CliError> {
        let cwd = env::current_dir()?;
        let args = [
            "install",
            &format!("{}=={}", dependency.name, dependency.version),
        ];
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
