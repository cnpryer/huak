use crate::errors::CliError;
use crate::errors::CliResult;
use huak::env::venv::create_venv;
use huak::ops;
use huak::package::installer::Installer;
use huak::project::Project;
use std::env;
use std::process::ExitCode;

/// Run the `test` command.
// TODO: Use pyproject.toml for configuration overrides.
pub fn run() -> CliResult<()> {
    // This command runs from the context of the cwd.
    let cwd = env::current_dir()?;
    let project = match Project::from_directory(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };
    let py_env = create_venv(project.root())
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;
    let installer = Installer::new();

    ops::test::test_project(&project, &py_env, &installer)
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    Ok(())
}
