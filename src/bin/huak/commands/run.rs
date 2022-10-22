use std::env;
use std::process::ExitCode;

use crate::errors::{CliError, CliResult};
use huak::ops;
use huak::project::Project;

/// Run the `run` command.
pub fn run(command: Vec<String>) -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project =
        Project::from(cwd).map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    ops::run::run_command(&project, &command)
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    Ok(())
}
