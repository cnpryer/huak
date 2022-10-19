use std::env;
use std::process::ExitCode;

use crate::errors::{CliError, CliResult};
use git2::Repository;
use huak::errors::HuakError;
use huak::ops;
use huak::project::Project;
use huak::project::ProjectType;

/// Run the `new` command.
pub fn run(path: String, app: bool, lib: bool, no_vcs: bool) -> CliResult<()> {
    // This command runs from in the context of the current working directory
    let cwd = env::current_dir()?;

    let project_type = match (app, lib) {
        (true, false) => ProjectType::Application,
        (false, true) => ProjectType::Library,
        _ => Default::default(),
    };

    // create PathBuf from `path` command line arg
    let path = cwd.join(path);

    // Check there isn't already a path we would override.
    if path.exists() && path != cwd {
        return Err(CliError::new(
            HuakError::DirectoryExists(path),
            ExitCode::FAILURE,
        ));
    }

    // If the current directory is used it must be empty. Otherwise, user should use `init`.
    if path == cwd && path.read_dir()?.count() > 0 {
        return Err(CliError::new(
            HuakError::DirectoryExists(path),
            ExitCode::FAILURE,
        ));
    }

    let project = Project::new(path, project_type);

    ops::new::create_project(&project)
        .map_err(|e| CliError::new(e, ExitCode::FAILURE))?;

    if !no_vcs {
        // Initialize git repository in the new project
        Repository::init(&project.root).map_err(|e| {
            CliError::new(HuakError::GitError(e), ExitCode::FAILURE)
        })?;
    }

    Ok(())
}
