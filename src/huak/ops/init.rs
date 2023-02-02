use crate::{
    errors::{HuakError, HuakResult},
    project::Project,
};

/// Initialize a project by adding a pyproject.toml to the dir.
// TODO: Do we need to mutate here?
pub fn init_project(project: &mut Project) -> HuakResult<()> {
    project.init_project_file()?;

    if let Some(path) = &project.project_file.filepath {
        if path.exists() {
            return Err(HuakError::PyProjectTomlExistsError);
        }
    }

    project.project_file.serialize()
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::project::ProjectType;

    use super::*;

    // TODO: --lib/--app
    #[test]
    fn toml() {
        let directory = tempdir().unwrap().into_path();
        let mut project = Project::new(directory, ProjectType::default());

        init_project(&mut project).unwrap();

        assert!(project.root().join("pyproject.toml").exists());
    }
}
