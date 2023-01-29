use std::{env, process::ExitCode};

use crate::errors::{CliError, CliResult};
use huak::{errors::HuakError, ops, project::Project};

/// Run the `version` command.
pub fn run() -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project = match Project::from_directory(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    let version = ops::version::get_project_version(&project)
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    if let Some(it) = project.project_file.project_name() {
        println!("{it}-{version}");
    } else {
        return Err(CliError::new(
            HuakError::PyProjectFileNotFound,
            ExitCode::FAILURE,
        ));
    }

    Ok(())
}
