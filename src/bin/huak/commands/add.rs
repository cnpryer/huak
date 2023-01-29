use std::env;
use std::process::ExitCode;
use std::str::FromStr;

use crate::errors::{CliError, CliResult};
use huak::env::venv::create_venv;
use huak::ops;
use huak::package::installer::Installer;
use huak::package::PythonPackage;
use huak::project::Project;

pub fn run(dependency: String, group: Option<String>) -> CliResult<()> {
    let python_package = &mut PythonPackage::from_str(&dependency)
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    let cwd = env::current_dir()?;
    let project = match Project::from_directory(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };
    let venv = create_venv(project.root())
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    let installer = Installer::new();

    ops::add::add_project_dependency(
        python_package,
        &project,
        &venv,
        &installer,
        group,
    )
    .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    Ok(())
}
