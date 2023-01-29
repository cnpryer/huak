use std::fs;

use crate::{
    errors::{HuakError, HuakResult},
    project::Project,
};

/// Initialize a project by adding a pyproject.toml to the dir.
pub fn init_project(project: &Project) -> HuakResult<()> {
    // Create a toml setting the name to the project directory's name.
    // TODO: Don't do this with a utility function.
    let toml = project.create_toml()?;

    if project.root().join("pyproject.toml").exists() {
        return Err(HuakError::PyProjectTomlExistsError);
    }

    // Serialize pyproject.toml.
    let string = toml.to_string()?;

    Ok(fs::write(project.root().join("pyproject.toml"), string)?)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    // TODO
    #[test]
    fn toml() {
        let directory = tempdir().unwrap().into_path();
        let project = Project::from(directory).unwrap();

        init_project(&project).unwrap();

        assert!(project.root().join("pyproject.toml").exists());
    }
}
