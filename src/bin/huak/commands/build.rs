use std::{env, process::ExitCode};

use huak::{env::venv::create_venv, ops, project::Project};

use crate::errors::{CliError, CliResult};

/// Run the `build` command.
pub fn run() -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project = match Project::from_directory(cwd) {
        Ok(it) => it,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };
    let venv = create_venv(project.root())
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    ops::build::build_project(&project, &venv)
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    Ok(())
}
