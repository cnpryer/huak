use crate::errors::{CliError, CliResult};
use std::env;
use std::process::ExitCode;

use huak::project::Project;

/// Run the `activate` command.
pub fn run() -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project = match Project::from(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    let venv = project
        .venv()
        .as_ref()
        .expect("`Project::from` creates venv if it doesn't exists.");

    println!("Venv activated: {}", venv.path.display());

    venv.activate()
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    println!("Venv deactivated.");

    Ok(())
}
