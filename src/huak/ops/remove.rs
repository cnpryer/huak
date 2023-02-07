use crate::{
    env::python_environment::Venv, errors::HuakResult,
    package::installer::Installer, project::Project,
};

/// Remove a dependency from a project by uninstalling it and updating the
/// project's config.
// TODO: Don't mutate
pub fn remove_project_dependency(
    project: &mut Project,
    py_env: &Venv,
    dependency: &str,
    installer: &Installer,
    group: &Option<String>,
) -> HuakResult<()> {
    installer.uninstall_package(dependency, py_env)?;

    project.project_file.remove_dependency(dependency, group)?;
    project.project_file.serialize()
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use crate::{
        env::python_environment::PythonEnvironment, package::PythonPackage,
        utils::test_utils::create_mock_project_full,
    };

    use super::*;

    #[test]
    fn removes_dependencies() {
        // TODO: Optional deps test is passing but the operation wasn't fully
        //       implemented.
        let mut project = create_mock_project_full().unwrap();
        let cwd = std::env::current_dir().unwrap();
        let venv = Venv::new(&cwd.join(".venv"));
        let installer = Installer::new();
        // TODO: #123 - destruction/deconstruction
        let reinstall_isort = venv.module_path("isort").unwrap().exists();
        let reinstall_pytest = venv.module_path("isort").unwrap().exists();
        install_pkg("pytest==7.2.1", &installer, &venv);
        install_pkg("isort==5.12.0", &installer, &venv);

        let isort_dep_existed = &project
            .project_file
            .dependency_list()
            .unwrap()
            .iter()
            .any(|d| d.starts_with("isort"));
        let isort_pkg_existed = venv.module_path("isort").unwrap().exists();
        let pytest_dep_existed = &project
            .project_file
            .optional_dependency_list("test")
            .map_or(false, |deps| deps.iter().any(|d| d.starts_with("pytest")));
        let pytest_pkg_existed = venv.module_path("pytest").unwrap().exists();

        remove_project_dependency(
            &mut project,
            &venv,
            "isort",
            &installer,
            &None,
        )
        .unwrap();
        remove_project_dependency(
            &mut project,
            &venv,
            "pytest",
            &installer,
            &Some("test".to_string()),
        )
        .unwrap();

        let isort_dep_exists = &project
            .project_file
            .dependency_list()
            .unwrap()
            .iter()
            .any(|s| s.starts_with("isort"));
        let isort_pkg_exists = venv.module_path("isort").unwrap().exists();
        let pytest_dep_exists = &project
            .project_file
            .optional_dependency_list("test")
            .map_or(false, |deps| deps.iter().any(|d| d.starts_with("pytest")));
        let pytest_pkg_exists = venv.module_path("pytest").unwrap().exists();

        assert!(isort_dep_existed);
        assert!(isort_pkg_existed);
        assert!(pytest_dep_existed);
        assert!(pytest_pkg_existed);
        assert!(!isort_dep_exists);
        assert!(!isort_pkg_exists);
        assert!(!pytest_dep_exists);
        assert!(!pytest_pkg_exists);

        if reinstall_isort {
            install_pkg("isort==5.12.0", &installer, &venv);
        }
        if reinstall_pytest {
            install_pkg("pytest==7.2.1", &installer, &venv);
        }
    }

    fn install_pkg(pkg_str: &str, installer: &Installer, py_env: &Venv) {
        installer
            .install_package(
                &PythonPackage::from_str(pkg_str).unwrap(),
                &py_env,
            )
            .unwrap();
    }
}
