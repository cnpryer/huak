use std::fs::remove_dir_all;

use crate::{
    errors::{CliError, CliResult},
    project::Project,
};

/// Clean build artifacts from a `Project`.
pub fn clean_project(project: &Project) -> CliResult<()> {
    // Just find dist at project root.
    let dist_path = project.root.join("dist");

    // If it's there delete it, otherwise just return Ok.
    if !dist_path.is_dir() {
        Ok(())
    } else {
        match remove_dir_all(dist_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(CliError::from(anyhow::format_err!(e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{env, path::PathBuf};

    use tempfile::tempdir;

    use crate::utils::{path::copy_dir, test_utils::create_mock_project};

    #[test]
    pub fn clean() {
        let directory = tempdir().unwrap().into_path().to_path_buf();
        let from_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("mock-project");

        copy_dir(&from_dir, &directory);

        let project =
            create_mock_project(directory.join("mock-project")).unwrap();
        let had_dist = project.root.join("dist").exists();

        clean_project(&project).unwrap();

        assert!(had_dist);
        assert!(project.root.as_path().exists());
        assert!(project.root.as_path().join("pyproject.toml").exists());
        assert!(project
            .root
            .as_path()
            .join("src")
            .join("mock_project")
            .join("__init__.pyc")
            .exists());
    }
}
