use std::env;
use std::process::ExitCode;

use crate::errors::{CliError, CliResult};
use huak::env::venv::create_venv;
use huak::ops;
use huak::package::installer::Installer;
use huak::project::Project;

/// Run the `remove` command.
pub fn run(dependency: String, group: Option<String>) -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project = match Project::from_directory(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };
    let venv = create_venv(project.root())
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;
    let installer = Installer::new();

    ops::remove::remove_project_dependency(
        &project,
        &venv,
        &dependency,
        &installer,
        &group,
    )
    .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    Ok(())
}
