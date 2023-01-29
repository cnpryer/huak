use std::env;
use std::process::ExitCode;

use crate::errors::{CliError, CliResult};
use huak::ops;
use huak::project::{Project, ProjectType};

/// Run the `init` command.
pub fn run(is_app: bool) -> CliResult<()> {
    let cwd = env::current_dir()?;

    let project_type = match is_app {
        true => ProjectType::Application,
        false => ProjectType::Library,
    };

    let mut project = Project::new(cwd, project_type);

    ops::init::init_project(&mut project)
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    Ok(())
}
