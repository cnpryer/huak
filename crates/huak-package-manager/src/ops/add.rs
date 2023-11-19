use crate::{dependency_iter, Config, Dependency, HuakResult, InstallOptions};
use pep440_rs::VersionSpecifiers;
use pep508_rs::VersionOrUrl;
use std::str::FromStr;

pub struct AddOptions {
    pub install_options: InstallOptions,
}

pub fn add_project_dependencies(
    dependencies: &[String],
    config: &Config,
    options: &AddOptions,
) -> HuakResult<()> {
    let workspace = config.workspace();
    let mut manifest = workspace.current_local_manifest()?;

    // Collect all dependencies that need to be added to the manifest file.
    let mut deps = dependency_iter(dependencies)
        .filter(|dep| {
            !manifest
                .manifest_data()
                .contains_project_dependency(dep.name())
        })
        .collect::<Vec<_>>();

    if deps.is_empty() {
        return Ok(());
    }

    let python_env = workspace.resolve_python_environment()?;
    python_env.install_packages(&deps, &options.install_options, config)?;

    // If there's no version data then get the installed version and add to manifest file.
    let packages = python_env.installed_packages()?; // TODO: Only run if versions weren't provided.
    for dep in &mut deps {
        if dep.requirement().version_or_url.is_none() {
            // TODO: Optimize this .find
            if let Some(pkg) = packages.iter().find(|p| p.name() == dep.name()) {
                dep.requirement_mut().version_or_url = Some(VersionOrUrl::VersionSpecifier(
                    VersionSpecifiers::from_str(&format!("=={}", pkg.version()))
                        .expect("package should have a version"),
                ));
            }
        }

        if !manifest
            .manifest_data()
            .contains_project_dependency(dep.name())
        {
            manifest
                .manifest_data_mut()
                .add_project_dependency(&dep.to_string());
        }
    }

    manifest.manifest_data_mut().formatted();
    manifest.write_file()?;

    Ok(())
}

pub fn add_project_optional_dependencies(
    dependencies: &[String],
    group: &str,
    config: &Config,
    options: &AddOptions,
) -> HuakResult<()> {
    let workspace = config.workspace();
    let mut manifest = workspace.current_local_manifest()?;

    // Collect all dependencies that need to be added.
    // TODO(cnpryer): Allow
    let mut deps = dependency_iter(dependencies)
        .filter(|dep| {
            !manifest
                .manifest_data()
                .contains_project_optional_dependency(dep.name(), group)
        })
        .collect::<Vec<Dependency>>();

    if deps.is_empty() {
        return Ok(());
    };

    let python_env = workspace.resolve_python_environment()?;
    python_env.install_packages(&deps, &options.install_options, config)?;

    // If there's no version data then get the installed version and add to manifest file.
    let packages = python_env.installed_packages()?; // TODO: Only run if versions weren't provided.
    for dep in &mut deps {
        if dep.requirement().version_or_url.is_none() {
            // TODO: Optimize this .find
            if let Some(pkg) = packages.iter().find(|p| p.name() == dep.name()) {
                dep.requirement_mut().version_or_url = Some(VersionOrUrl::VersionSpecifier(
                    VersionSpecifiers::from_str(&format!("=={}", pkg.version()))
                        .expect("package should have a version"),
                ));
            }
        }

        if !manifest
            .manifest_data()
            .contains_project_optional_dependency(dep.name(), group)
        {
            manifest
                .manifest_data_mut()
                .add_project_optional_dependency(&dep.to_string(), group);
        }
    }

    manifest.manifest_data_mut().formatted();
    manifest.write_file()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{copy_dir, initialize_venv, CopyDirOptions, TerminalOptions, Verbosity};
    use huak_dev::dev_resources_dir;
    use tempfile::tempdir;

    #[test]
    fn test_add_project_dependencies() {
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
        let venv = ws.resolve_python_environment().unwrap();
        let options = AddOptions {
            install_options: InstallOptions { values: None },
        };

        add_project_dependencies(&[String::from("ruff")], &config, &options).unwrap();

        let dep = Dependency::from_str("ruff").unwrap();
        let manifest = ws.current_local_manifest().unwrap();

        assert!(venv.contains_module("ruff").unwrap());
        assert!(manifest
            .manifest_data()
            .contains_project_dependency(dep.name()));
    }

    #[test]
    fn test_add_optional_project_dependencies() {
        let dir = tempdir().unwrap();
        copy_dir(
            &dev_resources_dir().join("mock-project"),
            &dir.path().join("mock-project"),
            &CopyDirOptions::default(),
        )
        .unwrap();
        let group = "dev";
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
        let venv = ws.resolve_python_environment().unwrap();
        let options = AddOptions {
            install_options: InstallOptions { values: None },
        };

        add_project_optional_dependencies(&[String::from("isort")], group, &config, &options)
            .unwrap();

        let dep = Dependency::from_str("isort").unwrap();
        let manifest = ws.current_local_manifest().unwrap();

        assert!(venv.contains_module("isort").unwrap());
        assert!(manifest
            .manifest_data()
            .contains_project_optional_dependency(dep.name(), "dev"));
    }
}
