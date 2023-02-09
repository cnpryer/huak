use std::{env, process::ExitCode};

use huak::{
    env::python_environment::create_venv, ops, package::installer::Installer,
    project::Project,
};

use crate::errors::{CliError, CliResult};

/// Run the `publish` command.
pub fn run() -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project = match Project::from_directory(cwd) {
        Ok(it) => it,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };
    let py_env = create_venv(project.root())
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    let installer = Installer::new();

    ops::publish::publish_project(&project, &py_env, &installer)
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))
}
