use std::{env, process::ExitCode};

use huak::{
    errors::{CliError, CliResult},
    ops,
    project::Project,
};

/// Run the `build` command.
pub fn run() -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project = match Project::from(cwd) {
        Ok(it) => it,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    if let Err(e) = ops::build::build_project(&project) {
        return Err(CliError::new(e, ExitCode::FAILURE));
    };

    Ok(())
}
