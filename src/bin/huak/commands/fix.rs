use crate::errors::CliError;
use huak::env::python_environment::create_venv;
use huak::ops;
use huak::package::installer::Installer;
use huak::project::Project;
use std::env;
use std::process::ExitCode;

use crate::errors::CliResult;

/// Run the `fix` command.
pub fn run() -> CliResult<()> {
    // This command runs from the context of the cwd.
    let cwd = env::current_dir()?;
    let project = match Project::from_directory(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };
    let py_venv = create_venv(project.root())
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;
    let installer = Installer::new();

    if let Err(e) = ops::fix::fix_project(&project, &py_venv, &installer) {
        return Err(CliError::new(e, ExitCode::FAILURE));
    };

    Ok(())
}
