use crate::errors::{CliError, CliResult};
use huak::{ops, project::Project};
use std::{env, process::ExitCode};

/// Run the `fmt` command.
pub fn run(is_check: bool) -> CliResult<()> {
    // This command runs from the context of the cwd.
    let cwd = env::current_dir()?;
    let project = match Project::from(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    if let Err(e) = ops::fmt::fmt_project(&project, &is_check) {
        return Err(CliError::new(e, ExitCode::FAILURE));
    };

    Ok(())
}
