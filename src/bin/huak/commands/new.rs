use std::env;
use std::fs;
use std::process::ExitCode;

use huak::errors::{CliError, CliResult, HuakError};
use huak::ops;
use huak::project::Project;

/// Run the `new` command.
// TODO: Ops should hanlde the path creation step in addition to the project creation.
pub fn run(path: Option<String>) -> CliResult<()> {
    // This command runs from the current working directory
    // Each command's behavior is triggered from the context of the cwd.
    let cwd = env::current_dir()?;

    // If a user passes a path
    let path = match path {
        Some(p) => cwd.join(p),
        _ => cwd.clone(),
    };

    // Make sure there isn't already a path we would override.
    if path.exists() && path != cwd {
        return Err(CliError::new(
            HuakError::DirectoryExists,
            ExitCode::FAILURE,
        ));
    }

    // If the current directory is used it must be empty. User should use init.
    if path == cwd && path.read_dir()?.count() > 0 {
        return Err(CliError::new(
            HuakError::DirectoryExists,
            ExitCode::FAILURE,
        ));
    }

    // Create project directory.
    if path != cwd {
        fs::create_dir_all(&path)?;
    }

    let project = Project::new(path);

    if let Err(e) = ops::new::create_project(&project) {
        return Err(CliError::new(e, ExitCode::FAILURE));
    };

    Ok(())
}
