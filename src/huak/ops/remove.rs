use std::fs;

use crate::{
    config::pyproject::toml::Toml,
    project::{python::PythonProject, Project},
};

/// Remove a dependency from a project by uninstalling it and updating the
/// project's config.
pub fn remove_project_dependency(
    project: &Project,
    dependency: &str,
) -> Result<(), anyhow::Error> {
    let venv = project.venv();

    // TODO: #109
    if let Err(e) = venv.uninstall_package(dependency) {
        return Err(anyhow::format_err!(e));
    };

    let mut toml = Toml::open(&project.root.join("pyproject.toml"))?;
    toml.project
        .dependencies
        .retain(|s| !s.starts_with(dependency));

    // Serialize pyproject.toml.
    fs::write(&project.root.join("pyproject.toml"), toml.to_string()?)?;

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
        let prev = toml
            .project
            .dependencies
            .into_iter()
            .filter(|s| s.starts_with("click"))
            .collect::<Vec<String>>();

        remove_project_dependency(&project, "click").unwrap();

        let toml = Toml::open(&toml_path).unwrap();
        let curr = toml
            .project
            .dependencies
            .into_iter()
            .filter(|s| s.starts_with("click"))
            .collect::<Vec<String>>();

        assert!(!prev.is_empty());
        assert!(curr.is_empty());
    }
}
