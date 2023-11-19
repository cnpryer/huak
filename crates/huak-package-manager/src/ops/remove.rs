use crate::{dependency_iter, Config, Error, HuakResult, InstallOptions};

pub struct RemoveOptions {
    pub install_options: InstallOptions,
}

pub fn remove_project_dependencies(
    dependencies: &[String],
    config: &Config,
    options: &RemoveOptions,
) -> HuakResult<()> {
    let workspace = config.workspace();
    let mut manifest = workspace.current_local_manifest()?;

    // Collect any dependencies to remove from the manifest file.
    let deps = dependency_iter(dependencies)
        .filter(|dep| {
            manifest
                .manifest_data()
                .contains_project_dependency_any(dep.name())
        })
        .collect::<Vec<_>>();

    if deps.is_empty() {
        return Ok(());
    }

    let optional_groups = manifest
        .manifest_data()
        .project_optional_dependency_groups();

    for dep in &deps {
        manifest
            .manifest_data_mut()
            .remove_project_dependency(dep.name());

        if let Some(groups) = optional_groups.as_ref() {
            for g in groups {
                manifest
                    .manifest_data_mut()
                    .remove_project_optional_dependency(dep.name(), g);
            }
        }
    }

    manifest.manifest_data_mut().formatted();
    manifest.write_file()?;

    // Uninstall the dependencies from the Python environment if an environment is found.
    match workspace.current_python_environment() {
        Ok(it) => it.uninstall_packages(&deps, &options.install_options, config),
        Err(Error::PythonEnvironmentNotFound) => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        copy_dir, initialize_venv, CopyDirOptions, Dependency, Package, TerminalOptions, Verbosity,
    };
    use huak_dev::dev_resources_dir;
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn test_remove_project_dependencies() {
        let dir = tempdir().unwrap();
        copy_dir(
            &dev_resources_dir().join("mock-project"),
            &dir.path().join("mock-project"),
            &CopyDirOptions::default(),
        )
        .unwrap();
        let workspace_root = dir.path().join("mock-project");
        let cwd = workspace_root.clone();
        let terminal_options = TerminalOptions {
            verbosity: Verbosity::Quiet,
            ..Default::default()
        };
        let config = Config {
            workspace_root,
            cwd,
            terminal_options,
            ..Default::default()
        };
        let options = RemoveOptions {
            install_options: InstallOptions { values: None },
        };
        let ws = config.workspace();
        initialize_venv(ws.root().join(".venv"), &ws.environment()).unwrap();
        let venv = ws.resolve_python_environment().unwrap();
        let test_package = Package::from_str("click==8.1.3").unwrap();
        let test_dep = Dependency::from_str("click==8.1.3").unwrap();
        venv.install_packages(&[&test_dep], &options.install_options, &config)
            .unwrap();
        let manifest = ws.current_local_manifest().unwrap();
        let venv_had_package = venv.contains_package(&test_package);
        let toml_had_package = manifest
            .manifest_data()
            .contains_project_dependency(test_dep.name());

        remove_project_dependencies(&["click".to_string()], &config, &options).unwrap();

        let ws = config.workspace();
        let manifest = ws.current_local_manifest().unwrap();
        let venv_contains_package = venv.contains_package(&test_package);
        let toml_contains_package = manifest
            .manifest_data()
            .contains_project_dependency(test_dep.name());

        assert!(venv_had_package);
        assert!(toml_had_package);
        assert!(!venv_contains_package);
        assert!(!toml_contains_package);
    }

    #[test]
    fn test_remove_project_optional_dependencies() {
        let dir = tempdir().unwrap();
        copy_dir(
            &dev_resources_dir().join("mock-project"),
            &dir.path().join("mock-project"),
            &CopyDirOptions::default(),
        )
        .unwrap();
        let workspace_root = dir.path().join("mock-project");
        let cwd = workspace_root.clone();
        let terminal_options = TerminalOptions {
            verbosity: Verbosity::Quiet,
            ..Default::default()
        };
        let config = Config {
            workspace_root,
            cwd,
            terminal_options,
            ..Default::default()
        };
        let options = RemoveOptions {
            install_options: InstallOptions { values: None },
        };
        let ws = config.workspace();
        initialize_venv(ws.root().join(".venv"), &ws.environment()).unwrap();
        let manifest = ws.current_local_manifest().unwrap();
        let venv = ws.resolve_python_environment().unwrap();
        let test_dep = Dependency::from_str("ruff").unwrap();
        venv.install_packages(&[&test_dep], &options.install_options, &config)
            .unwrap();
        let venv_had_package = venv.contains_module(test_dep.name()).unwrap();
        let toml_had_package = manifest
            .manifest_data()
            .contains_project_optional_dependency(test_dep.name(), "dev");

        remove_project_dependencies(&["ruff".to_string()], &config, &options).unwrap();

        let ws = config.workspace();
        let manifest = ws.current_local_manifest().unwrap();
        let venv_contains_package = venv
            .contains_module(&manifest.manifest_data().project_name().unwrap().to_string())
            .unwrap();
        let toml_contains_package = manifest
            .manifest_data()
            .contains_project_dependency(test_dep.name());

        assert!(venv_had_package);
        assert!(toml_had_package);
        assert!(!venv_contains_package);
        assert!(!toml_contains_package);
    }
}
