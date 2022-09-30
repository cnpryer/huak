use std::env;
use std::process::ExitCode;

use huak::ops;
use huak::{
    errors::{CliError, CliResult},
    project::Project,
};

/// Run the `remove` command.
pub fn run(dependency: String) -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project = match Project::from(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    if let Err(e) =
        ops::remove::remove_project_dependency(&project, &dependency)
    {
        return Err(CliError::new(e, ExitCode::FAILURE));
    };

    Ok(())
}
