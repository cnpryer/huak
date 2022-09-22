use crate::{
    env::python::PythonEnvironment,
    errors::{CliError, CliResult},
    project::{config::PythonConfig, python::PythonProject, Project},
};

/// Install all of the projects defined dependencies.
pub fn install_project_dependencies(project: &Project) -> CliResult<()> {
    // TODO: Doing this venv handling seems hacky.
    if !project.root.join("pyproject.toml").exists() {
        return Err(CliError::from(
            anyhow::format_err!("No pyproject.toml found")
        ));
    }

    for dependency in &project.config().dependency_list("main") {
        project.venv().install_package(dependency)?;
    }

    for dependency in &project.config().dependency_list("dev") {
        project.venv().install_package(dependency)?;
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {

    use tempfile::tempdir;

    use crate::utils::{
        path::copy_dir,
        test_utils::{create_mock_project, get_resource_dir},
    };
    use crate::{
        env::python::PythonEnvironment, project::python::PythonProject,
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
        let venv = project.venv();

        venv.uninstall_package("black").unwrap();
        let black_path = venv.module_path("black").unwrap();
        let had_black = black_path.exists();

        install_project_dependencies(&project).unwrap();

        assert!(!had_black);
        assert!(venv.module_path("black").unwrap().exists());
    }
}
