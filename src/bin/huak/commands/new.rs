use std::env;
use std::fs;
use std::process::ExitCode;

use huak::errors::{CliError, CliResult, HuakError};
use huak::ops;
use huak::project::Project;
use huak::project::ProjectType;

/// Run the `new` command.
// TODO: Ops should handle the path creation step in addition to the project creation.
pub fn run(path: Option<String>, app: bool, lib: bool) -> CliResult<()> {
    // This command runs from the current working directory
    // Each command's behavior is triggered from the context of the cwd.
    let cwd = env::current_dir()?;

    // The user cannot ask for both kinds of project at once.
    if app && lib {
        return Err(CliError::new(
            HuakError::ConflictingArguments,
            ExitCode::FAILURE,
        ));
    }

    let project_type = match (app, lib) {
        (true, false) => ProjectType::Application,
        (false, true) => ProjectType::Library,
        _ => Default::default(),
    };

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

    let project = Project::new(path, project_type);

    if let Err(e) = ops::new::create_project(&project) {
        return Err(CliError::new(e, ExitCode::FAILURE));
    };

    Ok(())
}
