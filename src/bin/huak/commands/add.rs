use std::env;
use std::process::ExitCode;

use huak::errors::{CliError, CliResult};
use huak::ops;
use huak::project::Project;

pub fn run(dependency: String, is_dev: bool) -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project = match Project::from(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    if let Err(e) =
        ops::add::add_project_dependency(&project, &dependency, is_dev)
    {
        return Err(CliError::new(e, ExitCode::FAILURE));
    }

    Ok(())
}
