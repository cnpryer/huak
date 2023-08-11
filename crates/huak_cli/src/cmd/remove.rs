use huak_ops::{dependency_iter, Config, Error, HuakResult, InstallOptions};

pub struct RemoveOptions {
    pub install_options: InstallOptions,
}

pub fn remove_project_dependencies(
    dependencies: &[String],
    config: &Config,
    options: &RemoveOptions,
) -> HuakResult<()> {
    let workspace = config.workspace();
    let package = workspace.current_package()?;
    let mut metadata = workspace.current_local_metadata()?;

    // Collect any dependencies to remove from the metadata file.
    let deps = dependency_iter(dependencies)
        .filter(|dep| {
            metadata
                .metadata()
                .contains_dependency_any(dep)
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();

    if deps.is_empty() {
        return Ok(());
    }

    // Get all groups from the metadata file to include in the removal process.
    let mut groups = Vec::new();
    if let Some(deps) = metadata.metadata().optional_dependencies() {
        groups.extend(deps.keys().map(|key| key.to_string()));
    }
    for dep in &deps {
        metadata.metadata_mut().remove_dependency(dep);
        for group in &groups {
            metadata
                .metadata_mut()
                .remove_optional_dependency(dep, group);
        }
    }

    if package.metadata() != metadata.metadata() {
        metadata.write_file()?;
    }

    // Uninstall the dependencies from the Python environment if an environment is found.
    match workspace.current_python_environment() {
        Ok(it) => {
            it.uninstall_packages(&deps, &options.install_options, config)
        }
        Err(Error::PythonEnvironmentNotFound) => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::test_fixtures::{
        test_config, test_resources_dir_path, test_venv,
    };
    use huak_ops::{copy_dir, CopyDirOptions, Dependency, Package, Verbosity};
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn test_remove_project_dependencies() {
        let dir = tempdir().unwrap();
        copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.path().join("mock-project"),
            &CopyDirOptions::default(),
        )
        .unwrap();
        let root = dir.path().join("mock-project");
        let cwd = root.to_path_buf();
        let config = test_config(&root, &cwd, Verbosity::Quiet);
        let options = RemoveOptions {
            install_options: InstallOptions { values: None },
        };
        let ws = config.workspace();
        test_venv(&ws);
        let venv = ws.resolve_python_environment().unwrap();
        let test_package = Package::from_str("click==8.1.3").unwrap();
        let test_dep = Dependency::from_str("click==8.1.3").unwrap();
        venv.install_packages(&[&test_dep], &options.install_options, &config)
            .unwrap();
        let metadata = ws.current_local_metadata().unwrap();
        let venv_had_package = venv.contains_package(&test_package);
        let toml_had_package =
            metadata.metadata().contains_dependency(&test_dep).unwrap();

        remove_project_dependencies(&["click".to_string()], &config, &options)
            .unwrap();

        let ws = config.workspace();
        let metadata = ws.current_local_metadata().unwrap();
        let venv_contains_package = venv.contains_package(&test_package);
        let toml_contains_package =
            metadata.metadata().contains_dependency(&test_dep).unwrap();

        assert!(venv_had_package);
        assert!(toml_had_package);
        assert!(!venv_contains_package);
        assert!(!toml_contains_package);
    }

    #[test]
    fn test_remove_project_optional_dependencies() {
        let dir = tempdir().unwrap();
        copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.path().join("mock-project"),
            &CopyDirOptions::default(),
        )
        .unwrap();
        let root = dir.path().join("mock-project");
        let cwd = root.to_path_buf();
        let config = test_config(&root, &cwd, Verbosity::Quiet);
        let options = RemoveOptions {
            install_options: InstallOptions { values: None },
        };
        let ws = config.workspace();
        test_venv(&ws);
        let metadata = ws.current_local_metadata().unwrap();
        let venv = ws.resolve_python_environment().unwrap();
        let test_package = Package::from_str("black==22.8.0").unwrap();
        let test_dep = Dependency::from_str("black==22.8.0").unwrap();
        venv.install_packages(&[&test_dep], &options.install_options, &config)
            .unwrap();
        let venv_had_package =
            venv.contains_module(test_package.name()).unwrap();
        let toml_had_package = metadata
            .metadata()
            .contains_optional_dependency(&test_dep, "dev")
            .unwrap();

        remove_project_dependencies(&["black".to_string()], &config, &options)
            .unwrap();

        let ws = config.workspace();
        let metadata = ws.current_local_metadata().unwrap();
        let venv_contains_package = venv
            .contains_module(metadata.metadata().project_name())
            .unwrap();
        let toml_contains_package =
            metadata.metadata().contains_dependency(&test_dep).unwrap();

        assert!(venv_had_package);
        assert!(toml_had_package);
        assert!(!venv_contains_package);
        assert!(!toml_contains_package);
    }
}
