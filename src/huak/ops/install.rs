use std::str::FromStr;

use crate::{
    env::venv::Venv,
    errors::{HuakError, HuakResult},
    package::{installer::Installer, PythonPackage},
    project::Project,
};

/// Install all of the projects defined dependencies.
pub fn install_project_dependencies(
    project: &Project,
    python_environment: &Venv,
    installer: &Installer,
    groups: &Option<Vec<String>>,
) -> HuakResult<()> {
    // TODO: Doing this venv handling seems hacky.
    if !project.root().join("pyproject.toml").exists() {
        return Err(HuakError::PyProjectFileNotFound);
    }

    installer.install_packages(
        &project
            .project_file
            .dependency_list()
            .iter()
            .filter_map(|d| PythonPackage::from_str(d).ok())
            .collect(),
        python_environment,
    )?;

    if let Some(them) = groups {
        for group in them {
            installer.install_packages(
                &project
                    .project_file
                    .optional_dependency_list(group)
                    .iter()
                    .filter_map(|d| PythonPackage::from_str(d).ok())
                    .collect(),
                python_environment,
            )?
        }
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {

    use crate::{
        env::venv::Venv, package::installer::Installer,
        utils::test_utils::create_mock_project_full,
    };

    use super::install_project_dependencies;

    // TODO
    #[test]
    fn installs_dependencies() {
        let mut project = create_mock_project_full().unwrap();
        project.init_project_file().unwrap();

        let cwd = std::env::current_dir().unwrap();
        let venv = &Venv::new(cwd.join(".venv"));
        let installer = Installer::new();

        venv.uninstall_package("black").unwrap();
        let black_path = venv.module_path("black").unwrap();
        let had_black = black_path.exists();

        venv.uninstall_package("pytest").unwrap();
        let pytest_path = venv.module_path("pytest").unwrap();
        let had_pytest = pytest_path.exists();

        install_project_dependencies(&project, &venv, &installer, &None)
            .unwrap();
        install_project_dependencies(
            &project,
            &venv,
            &installer,
            &Some(vec!["test".to_string()]),
        )
        .unwrap();

        assert!(!had_black);
        assert!(venv.module_path("black").unwrap().exists());
        assert!(!had_pytest);
        assert!(venv.module_path("pytest").unwrap().exists());
    }
}
