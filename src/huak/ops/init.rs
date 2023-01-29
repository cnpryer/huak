use crate::{errors::HuakResult, project::Project};

/// Initialize a project by adding a pyproject.toml to the dir.
pub fn init_project(project: &mut Project) -> HuakResult<()> {
    project.init_project_file()?;
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
