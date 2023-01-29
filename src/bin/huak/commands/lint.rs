use crate::errors::CliError;
use huak::env::venv::create_venv;
use huak::ops;
use huak::project::Project;
use std::env;
use std::process::ExitCode;

use crate::errors::CliResult;

/// Run the `lint` command.
pub fn run(fix: bool) -> CliResult<()> {
    // This command runs from the context of the cwd.
    let cwd = env::current_dir()?;
    let project = match Project::from_directory(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };
    let venv = create_venv(project.root())
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    let res = match fix {
        true => ops::fix::fix_project(&project, &venv),
        false => ops::lint::lint_project(&project, &venv),
    };

    res.map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    Ok(())
}
