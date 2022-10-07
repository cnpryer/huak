use std::env;
use std::process::ExitCode;

use crate::errors::{CliError, CliResult};
use huak::ops;
use huak::project::Project;

/// Run the `install` command.
pub fn run(groups: Option<Vec<String>>, all: bool) -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project = match Project::from(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    ops::install::install_project_dependencies(
        &project,
        &groups.unwrap_or_default(),
        all,
    )
    .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    Ok(())
}
