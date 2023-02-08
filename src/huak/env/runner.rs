use std::{
    env::{self, consts::OS, current_dir},
    path::{Path, PathBuf},
    process,
};

use crate::{
    errors::{HuakError, HuakResult},
    utils::{self, path, shell::get_shell_name},
};

use super::python_environment::{PythonEnvironment, Venv};

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
        let flag = match OS {
            "windows" => "/C",
            _ => "-c",
        };

        let program = get_shell_name()?;
        let mut path_var =
            format!("{}:", utils::path::to_string(&py_env.bin_path())?);
        path_var.push_str(&env::var("PATH")?);

        process::Command::new(program)
            .env("PATH", path_var)
            .args([flag, command])
            .current_dir(from.unwrap_or(&self.home))
            .spawn()?
            .wait()?;

        Ok(())
    }
}
