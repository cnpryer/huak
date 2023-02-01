use std::env;
use std::process::ExitCode;

use crate::errors::{CliError, CliResult};
use huak::env::venv::Venv;
use huak::errors::HuakError;
use huak::ops;
use huak::package::installer::Installer;
use huak::project::Project;

/// Run the `install` command.
pub fn run(groups: Option<Vec<String>>) -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project = match Project::from_directory(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    // Attempt to locate the project's venv. If none is found, attempt to create one.
    let py_env = match Venv::from_directory(project.root()) {
        Ok(it) => it,
        Err(HuakError::PyVenvNotFoundError) => {
            let it = Venv::new(project.root().join(".venv"));
            it.create()
                .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;
            it
        }
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    let installer = Installer::new();

    ops::install::install_project_dependencies(
        &project, &py_env, &installer, &groups,
    )
    .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    Ok(())
}
