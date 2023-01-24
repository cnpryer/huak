use std::env;
use std::process::ExitCode;

use crate::errors::{CliError, CliResult};
use huak::env::venv::Venv;
use huak::errors::HuakError;
use huak::ops;
use huak::project::Project;

/// Run the `install` command.
pub fn run(groups: Option<Vec<String>>, all: bool) -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project = match Project::from(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    // Attempt to locate the project's venv. If none is found, attempt to create one.
    let venv = match Venv::from_path(project.root()) {
        Ok(it) => it,
        Err(HuakError::VenvNotFound) => {
            let it = Venv::new(project.root().join(".venv"));
            it.create()
                .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;
            it
        }
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    ops::install::install_project_dependencies(
        &venv,
        &project,
        &groups.unwrap_or_default(),
        all,
    )
    .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    Ok(())
}
