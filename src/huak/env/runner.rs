use std::{
    env::{consts::OS, current_dir},
    path::{Path, PathBuf},
};

use crate::{
    errors::{HuakError, HuakResult},
    utils::{self, path, shell::get_shell_name},
};

use super::python_environment::{Activatable, PythonEnvironment, Venv};

#[derive(Default)]
pub struct Runner {
    /// The "from" directory of the Runner. "Home" is consider the
    /// original directory the Runner runs commands from.
    home: PathBuf,
}

impl Runner {
    pub fn new() -> HuakResult<Runner> {
        let home = current_dir()?;

        Ok(Runner { home })
    }

    /// Run a module installed to a valid Python environment.
    pub fn run_installed_module(
        &self,
        module: &str,
        args: &[&str],
        py_env: &Venv,
        from: Option<&Path>,
    ) -> HuakResult<()> {
        py_env.validate()?;

        let module_path = py_env.module_path(module)?;
        if !module_path.exists() {
            return Err(HuakError::PyModuleMissingError(module.to_string()));
        }

        crate::utils::command::run_command(
            path::to_string(module_path.as_path())?,
            args,
            from.unwrap_or(self.home.as_path()),
        )?;

        Ok(())
    }

    /// Run a command formatted as an &str from the context of a valid Python
    /// environment.
    pub fn run_str_command(
        &self,
        command: &str,
        py_env: &Venv,
        from: Option<&Path>,
    ) -> HuakResult<()> {
        py_env.validate()?;

        let script = py_env.get_activation_script_path()?;
        let activation_command = match OS {
            "windows" => utils::path::to_string(&script)?.to_string(),
            _ => format!("source {}", script.display()),
        };
        let program = get_shell_name()?;
        let program = if program.as_str() == "sh" {
            ".".to_string()
        } else {
            program
        };

        crate::utils::command::run_command(
            &program,
            &["-c", &format!("{activation_command} && {command}")],
            from.unwrap_or(self.home.as_path()),
        )?;

        Ok(())
    }
}
