use std::fs;

use crate::env::python::PythonEnvironment;
use crate::{
    config::pyproject::toml::Toml,
    project::{python::PythonProject, Project},
};

pub fn remove_project_dependency(
    project: &Project,
    dependency: &str,
) -> Result<(), anyhow::Error> {
    let venv = match project.venv() {
        Some(v) => v,
        _ => return Err(anyhow::format_err!("no venv found")),
    };

    // TODO: #109
    if let Err(e) = venv.uninstall_package(dependency) {
        return Err(anyhow::format_err!(e.error.unwrap()));
    };

    let mut toml = Toml::open(&project.root.join("pyproject.toml"))?;
    toml.tool.huak.dependencies.remove(dependency);
    toml.tool.huak.dev_dependencies.remove(dependency);

    // Serialize pyproject.toml.
    fs::write(&project.root.join("pyproject.toml"), toml.to_string()?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    use crate::utils::test_utils::{
        copy_dir, create_mock_project, get_resource_dir,
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
        let had_path = toml.tool.huak.dependencies.contains_key("click");

        remove_project_dependency(&project, "click").unwrap();

        let toml = Toml::open(&toml_path).unwrap();
        let has_path = toml.tool.huak.dependencies.contains_key("click");

        assert!(had_path);
        assert!(!has_path);
    }
}
