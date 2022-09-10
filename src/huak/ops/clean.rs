use std::fs::remove_dir_all;

use crate::{
    errors::{CliError, CliResult},
    project::Project,
};

pub fn clean_project(project: &Project) -> CliResult {
    if !project.root.join("dist").is_dir() {
        Ok(())
    } else {
        match remove_dir_all("dist") {
            Ok(_) => Ok(()),
            Err(e) => Err(CliError::new(anyhow::format_err!(e), 2)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use crate::{
        ops::clean::clean_project, project::Project,
        test_utils::create_mock_project_from_dir,
    };

    #[test]
    pub fn clean() {
        let directory = tempdir().unwrap().into_path().to_path_buf();
        let from_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("mock-project");

        create_mock_project_from_dir(&from_dir, &directory);
        let project = Project::new(directory.join("mock-project"));
        // let had_dist = project.root.join("dist").exists();

        let _ = clean_project(&project);

        // assert!(had_dist);
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
