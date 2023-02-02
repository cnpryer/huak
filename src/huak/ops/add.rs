use std::str::FromStr;

use crate::{
    env::venv::Venv,
    errors::HuakResult,
    package::{installer::Installer, PythonPackage},
    project::Project,
};

// TODO: Some of these comments are applicable to the package installation
//       process in general. This function is not considered robust yet.
//       Some missing features would include:
//         - Lockfiles
//         - Multiple version number dependency strings(?)
//         - Multiple optional dependeny groups
//
/// This function expects a reference to a valid `Project`, `Venv`
/// (TODO: Python Envrionment), and `PythonPackage`.
///
/// To add a dependency (Python Package) to the Python project the following
/// steps are occur:
///   1. If the dependency is provided without a PEP 440 version number, a
///      version number would need to be established. To do so:
///       a. If the dependency is already listed in the project file's
///          dependency list(s), then the version (if any) would be pulled
///          from the project file. If no version exists in the project file
///          for the dependency, then no version will be added to the
///          project file after installation (TODO: This may change).
///       b. If the user has not opted-out of the package installation cache,
///          the version number would be pulled from the latest version
///          found in the package installation cache the installer would use.
///       c. If the user has opted out of the package installation cache then
///          the version number would be established from the latest available
///          from the package index the installer would use.
///   2. Check if the dependency needs to be installed to the Python
///      environment provided. The dependency needs to be installed if:
///       a. The dependency is not already installed in the Python
///          environment provided or the package installation cache the
///          project would utilize (TODO). If the package is installed in
///          the package installation cache the project would use, and the
///          user did not configure the project to ignore the installation
///          cache, then the project would install from the cache.
///       b. The dependency is already installed to the package installation
///          cache the project would use, but the user opted out of using the
///          package installation cache (TODO).
///       c. The dependency is already installed but the version the user
///          provided is different from the one that has been installed. In
///          this case the installed package would need to be uninstalled from
///          the Python environment provide, and the new version of the package
///          would need to be installed. If the version provided is older than
///          the installed package version(s) then the user would need to be
///          told to retry with explicit instructions to downgrade (TODO).
///          TODO: Handle global installation behavior in this case.
///   3. Serialize the project file with any modifications that are necesary.
///      Modifications would be needed if:
///       a. The installed dependency includes a version that does not match
///          the version found from the project file prior to installation.
///       b. The group of the dependency has changed.
pub fn add_project_dependency(
    package: &PythonPackage,
    project: &mut Project,
    py_env: &Venv,
    installer: &Installer,
    dependency_group: Option<String>,
) -> HuakResult<()> {
    let (in_dependency_list, is_new_version);
    let package = match &project
        .project_file
        .search_dependency_list(package, &dependency_group)?
    {
        Some(it) => {
            in_dependency_list = true;
            let found = PythonPackage::from_str(it)?;
            if package.version() > found.version() {
                is_new_version = false;
                package.clone()
            } else {
                is_new_version = true;
                found
            }
        }
        None => {
            in_dependency_list = false;
            is_new_version = false;
            package.clone()
        }
    };

    installer.install_package(&package, py_env)?;

    let package = if package.version().is_none() {
        if let Some(it) = installer.last_installed_package()? {
            it
        } else {
            package
        }
    } else {
        package
    };

    if !in_dependency_list | is_new_version {
        match &dependency_group {
            Some(it) => {
                project
                    .project_file
                    .add_optional_dependency(&package.to_string(), it)?;
            }
            None => {
                project.project_file.add_dependency(&package.to_string())?;
            }
        }
    }

    project.project_file.serialize()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::test_utils::create_mock_project_full;

    #[test]
    fn add_dependency() {
        // TODO: Test optional dep.
        let mut project = create_mock_project_full().unwrap();
        project.init_project_file().unwrap();
        let cwd = std::env::current_dir().unwrap();
        let venv = Venv::new(cwd.join(".venv"));
        let installer = Installer::new();
        let package = PythonPackage::from_str("mock").unwrap();
        let reinstall_mock = venv.module_path("mock").unwrap().exists();

        installer.uninstall_package(&package.name, &venv).unwrap();

        let had_dep = project
            .project_file
            .dependency_list()
            .unwrap_or(&Vec::new())
            .iter()
            .any(|d| d.starts_with(&package.name));

        add_project_dependency(&package, &mut project, &venv, &installer, None)
            .unwrap();

        let has_dep = project
            .project_file
            .dependency_list()
            .unwrap_or(&Vec::new())
            .iter()
            .any(|d| d.starts_with(&package.name));

        assert!(!had_dep);
        assert!(has_dep);

        // TODO: #123 - destruction/deconstruction
        if reinstall_mock {
            installer.install_package(&package, &venv).unwrap();
        }
    }

    #[test]
    fn add_optional_dependency() {
        // TODO: Test optional dep.
        let mut project = create_mock_project_full().unwrap();
        project.init_project_file().unwrap();
        let cwd = std::env::current_dir().unwrap();
        let venv = Venv::new(cwd.join(".venv"));
        let installer = Installer::new();
        let package = PythonPackage::from_str("isort==5.12.0").unwrap();
        let reinstall_isort = venv.module_path("isort").unwrap().exists();

        installer.uninstall_package(&package.name, &venv).unwrap();

        let had_dep = project
            .project_file
            .optional_dependency_list("test")
            .unwrap_or(&Vec::new())
            .iter()
            .any(|d| d.starts_with(&package.name));

        add_project_dependency(
            &package,
            &mut project,
            &venv,
            &installer,
            Some("test".to_string()),
        )
        .unwrap();

        let has_dep = project
            .project_file
            .optional_dependency_list("test")
            .unwrap_or(&Vec::new())
            .iter()
            .any(|d| d.starts_with(&package.name));

        assert!(!had_dep);
        assert!(has_dep);

        // TODO: #123 - destruction/deconstruction
        if reinstall_isort {
            installer.install_package(&package, &venv).unwrap();
        }
    }
}
