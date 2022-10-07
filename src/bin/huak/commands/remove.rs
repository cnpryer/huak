use std::env;
use std::process::ExitCode;

use crate::errors::{CliError, CliResult};
use huak::ops;
use huak::project::Project;

/// Run the `remove` command.
pub fn run(dependency: String) -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project = match Project::from(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    ops::remove::remove_project_dependency(&project, &dependency)
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    Ok(())
}
