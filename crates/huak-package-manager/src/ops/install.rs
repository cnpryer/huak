use crate::{Config, HuakResult, InstallOptions};

pub fn install_project_dependencies(
    groups: Option<&Vec<String>>,
    config: &Config,
    options: &InstallOptions,
) -> HuakResult<()> {
    let workspace = config.workspace();
    let manifest = workspace.current_local_manifest()?;

    let mut dependencies = Vec::new();

    if let Some(gs) = groups {
        // If the group "required" is passed and isn't a valid optional dependency group
        // then install just the required dependencies.
        // TODO(cnpryer): Refactor/move
        if manifest
            .manifest_data()
            .project_optional_dependency_groups()
            .map_or(false, |it| it.iter().any(|s| s == "required"))
        {
            if let Some(reqs) = manifest.manifest_data().project_dependencies() {
                dependencies.extend(reqs);
            }
        } else if let Some(optional_deps) = manifest.manifest_data().project_optional_dependencies()
        {
            for g in gs {
                // TODO(cnpryer): Perf
                if let Some(deps) = optional_deps.get(&g.to_string()) {
                    dependencies.extend(deps.iter().cloned());
                }
            }
        }
    } else {
        // If no groups are passed then install all dependencies listed in the manifest file
        // including the optional dependencies.
        if let Some(reqs) = manifest.manifest_data().project_dependencies() {
            dependencies.extend(reqs);
        }

        // TODO(cnpryer): Install optional as opt-in
        if let Some(groups) = manifest
            .manifest_data()
            .project_optional_dependency_groups()
        {
            for key in groups {
                if let Some(g) = manifest.manifest_data().project_optional_dependencies() {
                    if let Some(it) = g.get(&key) {
                        dependencies.extend(it.iter().cloned());
                    }
                }
            }
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
    use super::*;
    use crate::{copy_dir, initialize_venv, CopyDirOptions, Package, TerminalOptions, Verbosity};
    use huak_dev::dev_resources_dir;
    use tempfile::tempdir;

    #[test]
    fn test_install_project_dependencies() {
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
