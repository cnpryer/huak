use std::fs;

use crate::{
    config::pyproject::toml::Toml, errors::HuakError, project::Project,
};

/// Remove a dependency from a project by uninstalling it and updating the
/// project's config.
pub fn remove_project_dependency(
    project: &Project,
    dependency: &str,
) -> Result<(), HuakError> {
    let venv = match project.venv() {
        Some(v) => v,
        _ => return Err(HuakError::VenvNotFound),
    };

    // TODO: #109
    if venv.uninstall_package(dependency).is_err() {
        return Err(HuakError::AnyHowError(anyhow::format_err!(
            "Failed to install {:?}",
            dependency
        )));
    };

    let mut toml = Toml::open(&project.root.join("pyproject.toml"))?;
    toml.project
        .dependencies
        .retain(|s| !s.starts_with(dependency));

    // Serialize pyproject.toml.
    let string = match toml.to_string() {
        Ok(s) => s,
        Err(_) => return Err(HuakError::IOError),
    };

    if fs::write(&project.root.join("pyproject.toml"), string).is_err() {
        return Err(HuakError::IOError);
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    use crate::utils::{
        path::copy_dir,
        test_utils::{create_mock_project, get_resource_dir},
    };

    #[test]
    fn removes_dependencies() {
        let directory = tempdir().unwrap().into_path().to_path_buf();
        let mock_project_path = get_resource_dir().join("mock-project");
        copy_dir(&mock_project_path, &directory);

        let project =
            create_mock_project(directory.join("mock-project")).unwrap();
        let toml_path = project.root.join("pyproject.toml");
        let toml = Toml::open(&toml_path).unwrap();
        let existed = toml
            .project
            .dependencies
            .iter()
            .any(|d| d.starts_with("click"));
        let existed = existed
            && toml
                .project
                .optional_dependencies
                .as_ref()
                .unwrap()
                .iter()
                .any(|d| d.starts_with("black"));

        remove_project_dependency(&project, "click").unwrap();

        let toml = Toml::open(&toml_path).unwrap();
        let exists = !toml
            .project
            .dependencies
            .iter()
            .any(|s| s.starts_with("black"));

        let exists = exists
            && toml
                .project
                .optional_dependencies
                .as_ref()
                .unwrap()
                .iter()
                .any(|s| s.starts_with("black"));

        assert!(existed);
        assert!(exists);
    }
}
