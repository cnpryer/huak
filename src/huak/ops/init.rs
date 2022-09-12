use std::fs;

use super::project_utils;
use crate::project::Project;

/// Initialize a project by adding a pyproject.toml to the dir.
pub fn init_project(project: &Project) -> Result<(), anyhow::Error> {
    // Create a toml setting the name to the project dir's name.
    // TODO: Don't do this with a utility function.
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
        let project = Project::from(directory).unwrap();

        init_project(&project).unwrap();

        assert!(project.root.join("pyproject.toml").exists());
    }
}
