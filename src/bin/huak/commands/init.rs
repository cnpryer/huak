use std::env;
use std::process::ExitCode;

use huak::errors::{CliError, CliResult};
use huak::ops;
use huak::project::Project;

/// Run the `init` command.
pub fn run() -> CliResult<()> {
    let cwd = env::current_dir()?;

    let project = match Project::from(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    if let Err(e) = ops::init::init_project(&project) {
        return Err(CliError::new(e, ExitCode::FAILURE));
    };

    Ok(())
}
