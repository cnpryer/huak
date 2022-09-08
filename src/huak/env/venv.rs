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
    pub fn find(from: &Path) -> Result<Venv, anyhow::Error> {
        let paths = vec![from.join(".venv"), from.join("venv")];

        for path in &paths {
            if path.exists() {
                return Ok(Venv {
                    path: path.to_path_buf(),
                });
            }
        }

        Err(anyhow::format_err!("no venv found"))
    }

    /// Get the name of the Venv (ex: ".venv").
    pub fn name(&self) -> Result<&str, anyhow::Error> {
        let name = crate::utils::path::parse_filename(&self.path.as_path())?;

        Ok(name)
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

    /// Install a dependency to
    fn install_package(&self, dependency: &PythonPackage) -> Result<(), CliError> {
        let args = [
            "install",
            &format!("{}=={}", dependency.name, dependency.version),
        ];

        let cwd = env::current_dir()?;
        let pip_path = self.bin_path().join("pip");
        let pip_path = crate::utils::path::as_string(&pip_path.as_path())?;

        crate::utils::command::run_command(pip_path, &args, &cwd)?;

        Ok(())
    }
}
