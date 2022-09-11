use crate::{
    config::python::PythonConfig,
    env::{python::PythonEnvironment, venv::Venv},
    errors::{CliError, CliResult},
    project::{python::PythonProject, Project},
};

pub fn install_project_dependencies(project: &Project) -> CliResult {
    // TODO: Doing this venv handling seems hacky.
    let mut venv = &Venv::new(project.root.join(".venv"));
    if let Some(v) = project.venv() {
        venv = v
    } else {
        venv.create()?;
    }

    if !project.root.join("pyproject.toml").exists() {
        return Err(CliError::new(
            anyhow::format_err!("no pyproject.toml found"),
            2,
        ));
    }

    for dependency in project.config().dependencies() {
        venv.install_package(dependency)?;
    }

    for dependency in project.config().dev_dependencies() {
        venv.install_package(dependency)?;
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use std::env;

    use tempfile::tempdir;

    use crate::{
        env::python::PythonEnvironment,
        project::python::PythonProject,
        utils::test_utils::{
            copy_dir, create_mock_project, create_venv, get_resource_dir,
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
        let cwd = env::current_dir().unwrap();
        // TODO: Option and getters making it tricky. Probably doing something wrong.
        let testing_venv = create_venv(cwd.join(".venv")).unwrap();
        let venv = if let Some(v) = project.venv() {
            v
        } else {
            &testing_venv
        };

        venv.uninstall_package("black").unwrap();

        let black_path = venv.bin_path().join("black");
        let had_black = black_path.exists();

        install_project_dependencies(&project).unwrap();

        assert!(!had_black);
        assert!(black_path.exists());
    }
}
