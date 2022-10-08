use std::process::ExitCode;

use crate::errors::{CliError, CliResult};

use huak::{ops, project::Project};

/// Run the `activate` command.
pub fn run() -> CliResult<()> {
    let cwd = std::env::current_dir()?;
    let project = match Project::from(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    ops::activate::activate_project_venv(&project)
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;
    Ok(())
}
