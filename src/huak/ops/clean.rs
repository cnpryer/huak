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
        test_utils::create_py_project_sample,
    };

    #[test]
    pub fn clean() {
        let directory = tempdir().unwrap().into_path().to_path_buf();
        let from_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("mock-project");

        create_py_project_sample(&from_dir, &directory);
        let project = Project::new(directory.clone());
        // let had_dist = project.root.join("dist").exists();

        let _ = clean_project(&project);

        // assert!(had_dist);
        assert!(directory.as_path().join("mock-project").exists());
        assert!(directory
            .as_path()
            .join("mock-project")
            .join("pyproject.toml")
            .exists());
        assert!(directory
            .as_path()
            .join("mock-project")
            .join("src")
            .join("mock_project")
            .join("__init__.pyc")
            .exists());
    }
}
