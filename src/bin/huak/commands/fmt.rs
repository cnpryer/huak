use crate::errors::{CliError, CliResult};
use huak::{env::venv::create_venv, ops, project::Project};
use std::{env, process::ExitCode};

/// Run the `fmt` command.
pub fn run(is_check: bool) -> CliResult<()> {
    // This command runs from the context of the cwd.
    let cwd = env::current_dir()?;
    let project = match Project::from_directory(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };
    let venv = create_venv(project.root())
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    ops::fmt::fmt_project(&project, &venv, &is_check)
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    Ok(())
}
