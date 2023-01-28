use crate::{
    env::venv::Venv,
    errors::{HuakError, HuakResult},
    package::PythonPackage,
    project::{Project, PythonConfig},
};

/// Install all of the projects defined dependencies.
pub fn install_project_dependencies(
    venv: &Venv,
    project: &Project,
    groups: &Vec<String>,
    all: bool,
) -> HuakResult<()> {
    // TODO: Doing this venv handling seems hacky.
    if !project.root().join("pyproject.toml").exists() {
        return Err(HuakError::PyProjectTomlNotFound);
    }

    install_packages(&project.config().package_list(), venv)?;

    if !all {
        for group in groups {
            install_packages(
                &project.config().optional_package_list(group),
                venv,
            )?
        }
        return Ok(());
    }

    if let Some(deps) = &project
        .config()
        .pyproject_toml()
        .project
        .optional_dependencies
    {
        for group in deps.keys() {
            install_packages(
                &project.config().optional_package_list(group),
                venv,
            )?;
        }
    }

    Ok(())
}

fn install_packages(
    packages: &Vec<PythonPackage>,
    venv: &Venv,
) -> HuakResult<()> {
    for package in packages {
        venv.install_package(package)?;
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {

    use crate::{env::venv::Venv, utils::test_utils::create_mock_project_full};

    use super::install_project_dependencies;

    // TODO
    #[test]
    fn installs_dependencies() {
        let project = create_mock_project_full().unwrap();
        let cwd = std::env::current_dir().unwrap();
        let venv = &Venv::new(cwd.join(".venv"));

        venv.uninstall_package("black").unwrap();
        let black_path = venv.module_path("black").unwrap();
        let had_black = black_path.exists();

        venv.uninstall_package("pytest").unwrap();
        let pytest_path = venv.module_path("pytest").unwrap();
        let had_pytest = pytest_path.exists();

        install_project_dependencies(&venv, &project, &vec![], false).unwrap();
        install_project_dependencies(
            &venv,
            &project,
            &vec!["test".to_string()],
            false,
        )
        .unwrap();

        assert!(!had_black);
        assert!(venv.module_path("black").unwrap().exists());
        assert!(!had_pytest);
        assert!(venv.module_path("pytest").unwrap().exists());
    }
}
