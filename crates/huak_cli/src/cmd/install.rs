use huak_python_package_manager::{Config, Dependency, HuakResult, InstallOptions};

pub fn install_project_dependencies(
    groups: Option<&Vec<String>>,
    config: &Config,
    options: &InstallOptions,
) -> HuakResult<()> {
    let workspace = config.workspace();
    let package = workspace.current_package()?;
    let metadata = workspace.current_local_metadata()?;

    let binding = Vec::new(); // TODO
    let mut dependencies = Vec::new();

    if let Some(gs) = groups {
        // If the group "required" is passed and isn't a valid optional dependency group
        // then install just the required dependencies.
        if package
            .metadata()
            .optional_dependency_group("required")
            .is_none()
            && gs.contains(&"required".to_string())
        {
            if let Some(reqs) = package.metadata().dependencies() {
                dependencies.extend(reqs.iter().map(Dependency::from));
            }
        } else {
            for g in gs {
                package
                    .metadata()
                    .optional_dependency_group(g)
                    .unwrap_or(&binding)
                    .iter()
                    .for_each(|req| {
                        dependencies.push(Dependency::from(req));
                    });
            }
        }
    } else {
        // If no groups are passed then install all dependencies listed in the metadata file
        // including the optional dependencies.
        if let Some(reqs) = package.metadata().dependencies() {
            dependencies.extend(reqs.iter().map(Dependency::from));
        }
        if let Some(deps) = metadata.metadata().optional_dependencies() {
            deps.values().for_each(|reqs| {
                dependencies.extend(reqs.iter().map(Dependency::from).collect::<Vec<_>>());
            });
        }
    }

    dependencies.dedup();

    if dependencies.is_empty() {
        return Ok(());
    }

    let python_env = workspace.resolve_python_environment()?;
    python_env.install_packages(&dependencies, options, config)
}

#[cfg(test)]
mod tests {
    use crate::cmd::test_utils::test_resources_dir_path;

    use super::*;
    use huak_python_package_manager::{
        copy_dir, initialize_venv, CopyDirOptions, Package, TerminalOptions, Verbosity,
    };
    use tempfile::tempdir;

    #[test]
    fn test_install_project_dependencies() {
        let dir = tempdir().unwrap();
        copy_dir(
            &test_resources_dir_path().join("mock-project"),
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
        };
        let ws = config.workspace();
        initialize_venv(ws.root().join(".venv"), &ws.environment()).unwrap();
        let options = InstallOptions { values: None };
        let venv = ws.resolve_python_environment().unwrap();
        let test_package = Package::from_str("click==8.1.3").unwrap();
        let had_package = venv.contains_package(&test_package);

        install_project_dependencies(None, &config, &options).unwrap();

        assert!(!had_package);
        assert!(venv.contains_package(&test_package));
    }

    #[test]
    fn test_install_project_optional_dependencies() {
        let dir = tempdir().unwrap();
        copy_dir(
            &test_resources_dir_path().join("mock-project"),
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
        };
        let ws = config.workspace();
        initialize_venv(ws.root().join(".venv"), &ws.environment()).unwrap();
        let options = InstallOptions { values: None };
        let venv = ws.resolve_python_environment().unwrap();
        let had_package = venv.contains_module("pytest").unwrap();

        install_project_dependencies(Some(&vec![String::from("dev")]), &config, &options).unwrap();

        assert!(!had_package);
        assert!(venv.contains_module("pytest").unwrap());
    }
}
