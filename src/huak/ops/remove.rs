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
    venv.uninstall_package(dependency)?;

    let mut toml = Toml::open(&project.root.join("pyproject.toml"))?;
    toml.remove_dependency(dependency);

    // Serialize pyproject.toml.
    let string = toml.to_string()?;

    fs::write(&project.root.join("pyproject.toml"), string)?;

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
        // TODO: Optional deps test is passing but the operation wasn't fully
        //       implemented.
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
            && toml.project.optional_dependencies.map_or(false, |deps| {
                deps.values().flatten().any(|d| d.starts_with("pytest"))
            });

        remove_project_dependency(&project, "click").unwrap();

        let toml = Toml::open(&toml_path).unwrap();
        let exists = toml
            .project
            .dependencies
            .iter()
            .any(|s| s.starts_with("click"));

        let exists = exists
            && toml.project.optional_dependencies.map_or(false, |deps| {
                deps.values().flatten().any(|d| d.starts_with("pytest"))
            });

        assert!(existed);
        assert!(!exists);
    }
}
