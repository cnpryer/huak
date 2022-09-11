use crate::{
    env::python::PythonEnvironment,
    errors::{CliError, CliResult},
    project::{config::PythonConfig, python::PythonProject, Project},
};

pub fn install_project_dependencies(project: &Project) -> CliResult {
    // TODO: Doing this venv handling seems hacky.
    if !project.root.join("pyproject.toml").exists() {
        return Err(CliError::new(
            anyhow::format_err!("no pyproject.toml found"),
            2,
        ));
    }

    if let Some(venv) = project.venv() {
        for dependency in &project.config().dependency_list("main") {
            venv.install_package(dependency)?;
        }

        for dependency in &project.config().dependency_list("dev") {
            venv.install_package(dependency)?;
        }
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {

    use tempfile::tempdir;

    use crate::{
        env::python::PythonEnvironment,
        project::python::PythonProject,
        utils::test_utils::{copy_dir, create_mock_project, get_resource_dir},
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

        let mut had_black = false;

        if let Some(v) = venv {
            v.uninstall_package("black").unwrap();
            let black_path = v.bin_path().join("black");
            had_black = black_path.exists();
        }

        install_project_dependencies(&project).unwrap();

        assert!(!had_black);

        if let Some(v) = venv {
            assert!(v.bin_path().join("black").exists());
        }
    }
}
