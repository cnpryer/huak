use std::fs;

use super::project_utils;
use crate::project::Project;

pub fn create_project_toml(project: &Project) -> Result<(), anyhow::Error> {
    let toml = project_utils::create_toml(project)?;

    if project.root.join("pyproject.toml").exists() {
        return Err(anyhow::format_err!("a pyproject.toml already exists"));
    }

    // Serialize pyproject.toml.
    fs::write(&project.root.join("pyproject.toml"), toml.to_string()?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    // TODO
    #[test]
    fn toml() {
        let directory = tempdir().unwrap().into_path().to_path_buf();
        let project = Project::new(directory);

        create_project_toml(&project).unwrap();

        assert!(project.root.join("pyproject.toml").exists());
    }
}
