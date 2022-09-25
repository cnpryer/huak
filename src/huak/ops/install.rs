use crate::{
    errors::{CliError, CliResult, HuakError},
    project::{config::PythonConfig, python::PythonProject, Project},
};

/// Install all of the projects defined dependencies.
pub fn install_project_dependencies(project: &Project) -> CliResult<()> {
    // TODO: Doing this venv handling seems hacky.
    if !project.root.join("pyproject.toml").exists() {
        return Err(CliError::new(HuakError::PyProjectTomlNotFound, 1));
    }

    let venv = match project.venv() {
        Some(v) => v,
        _ => return Err(CliError::new(HuakError::VenvNotFound, 1)),
    };

    for dependency in &project.config().dependency_list() {
        venv.install_package(dependency)?;
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {

    use tempfile::tempdir;

    use crate::{
        project::python::PythonProject,
        utils::{
            path::copy_dir,
            test_utils::{create_mock_project, get_resource_dir},
        },
    };

    use super::install_project_dependencies;

    // TODO
    #[test]
    fn installs_dependenices() {
        let directory = tempdir().unwrap().into_path().to_path_buf();
        let mock_project_dir = get_resource_dir().join("mock-project");
        copy_dir(&mock_project_dir, &directory);

        let project_path = directory.join("mock-project");
        let project = create_mock_project(project_path.clone()).unwrap();
        let venv = project.venv().as_ref().unwrap();

        venv.uninstall_package("black").unwrap();
        let black_path = venv.module_path("black").unwrap();
        let had_black = black_path.exists();

        install_project_dependencies(&project).unwrap();

        assert!(!had_black);
        assert!(venv.module_path("black").unwrap().exists());
    }
}
