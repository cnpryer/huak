use std::process::ExitCode;

use crate::errors::{CliError, CliResult};

use huak::{env::venv::create_venv, ops, project::Project};

/// Run the `activate` command.
pub fn run() -> CliResult<()> {
    let cwd = std::env::current_dir()?;
    let project = match Project::from(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };
    let venv = create_venv(project.root())
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    ops::activate::activate_venv(&venv)
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;
    Ok(())
}
