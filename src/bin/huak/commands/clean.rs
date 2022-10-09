use std::{env, process::ExitCode};

use huak::{ops, project::Project};

use crate::errors::{CliError, CliResult};

/// Run the `clean` command.
pub fn run(pycache: bool) -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project = match Project::from(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    let res = match pycache {
        true => ops::clean::clean_project_pycache(),
        false => ops::clean::clean_project(&project),
    };

    res.map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    Ok(())
}
