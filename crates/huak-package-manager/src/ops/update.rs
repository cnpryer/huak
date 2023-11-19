use crate::{dependency_iter, Config, Dependency, HuakResult, InstallOptions};
use std::str::FromStr;

pub struct UpdateOptions {
    pub install_options: InstallOptions,
}

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::needless_pass_by_value)]
pub fn update_project_dependencies(
    dependencies: Option<Vec<String>>,
    config: &Config,
    options: &UpdateOptions,
) -> HuakResult<()> {
    let workspace = config.workspace();
    let mut metadata = workspace.current_local_metadata()?;
    let python_env = workspace.resolve_python_environment()?;

    // Collect dependencies to update if they are listed in the metadata file.
    if let Some(it) = dependencies.as_ref() {
        let deps = dependency_iter(it)
            .filter_map(|dep| {
                if metadata
                    .metadata()
                    .contains_project_dependency_any(dep.name())
                {
                    Some(dep)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if deps.is_empty() {
            return Ok(());
        }

        python_env.update_packages(&deps, &options.install_options, config)?;
    } else {
        let mut deps = metadata
            .metadata()
            .project_dependencies()
            .map_or(Vec::new(), |reqs| reqs.into_iter().collect::<Vec<_>>());

        if let Some(gs) = metadata.metadata().project_optional_dependency_groups() {
            if let Some(optional_deps) = metadata.metadata().project_optional_dependencies() {
                for g in gs {
                    // TODO(cnpryer): Perf
                    if let Some(it) = optional_deps.get(&g.to_string()) {
                        deps.extend(it.iter().cloned());
                    }
                }
            }
        }

        deps.dedup();

        python_env.update_packages(&deps, &options.install_options, config)?;
    }

    let groups = metadata.metadata().project_optional_dependency_groups();

    for pkg in python_env.installed_packages()? {
        let dep = &Dependency::from_str(&pkg.to_string())?;
        if metadata.metadata().contains_project_dependency(dep.name()) {
            metadata
                .metadata_mut()
                .remove_project_dependency(dep.name());
            metadata
                .metadata_mut()
                .add_project_dependency(&dep.to_string());
        }

        if let Some(gs) = groups.as_ref() {
            for g in gs {
                if metadata
                    .metadata()
                    .contains_project_optional_dependency(dep.name(), g)
                {
                    metadata
                        .metadata_mut()
                        .remove_project_optional_dependency(dep.name(), g);
                    metadata
                        .metadata_mut()
                        .add_project_optional_dependency(&dep.to_string(), g);
                }
            }
        }
    }

    metadata.metadata_mut().formatted();
    metadata.write_file()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{copy_dir, initialize_venv, CopyDirOptions, TerminalOptions, Verbosity};
    use huak_dev::dev_resources_dir;
    use tempfile::tempdir;

    #[test]
    fn test_update_project_dependencies() {
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
        let options = UpdateOptions {
            install_options: InstallOptions { values: None },
        };

        update_project_dependencies(None, &config, &options).unwrap();
    }

    #[test]
    fn test_update_project_optional_dependencies() {
        let dir = tempdir().unwrap();
        copy_dir(
            dev_resources_dir().join("mock-project"),
            dir.path().join("mock-project"),
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
        let options = UpdateOptions {
            install_options: InstallOptions { values: None },
        };

        update_project_dependencies(None, &config, &options).unwrap();
    }
}
