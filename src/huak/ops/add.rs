use crate::{
    dependency::{dependency_iter, Dependency},
    Config, HuakResult, InstallOptions,
};
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
    let package = workspace.current_package()?;
    let mut metadata = workspace.current_local_metadata()?;

    // Collect all dependencies that need to be added to the metadata file.
    let mut deps: Vec<Dependency> = dependency_iter(dependencies)
        .filter(|dep| {
            !metadata
                .metadata()
                .contains_dependency(dep)
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();

    if deps.is_empty() {
        return Ok(());
    }

    let python_env = workspace.resolve_python_environment()?;
    python_env.install_packages(&deps, &options.install_options, config)?;

    // If there's no version data then get the installed version and add to metadata file.
    let packages = python_env.installed_packages()?; // TODO: Only run if versions weren't provided.
    for dep in deps.iter_mut() {
        if dep.requirement().version_or_url.is_none() {
            // TODO: Optimize this .find
            if let Some(pkg) = packages.iter().find(|p| p.name() == dep.name())
            {
                dep.requirement_mut().version_or_url =
                    Some(VersionOrUrl::VersionSpecifier(
                        VersionSpecifiers::from_str(&format!(
                            "=={}",
                            pkg.version()
                        ))
                        .expect("package should have a version"),
                    ));
            }
        }

        if !metadata.metadata().contains_dependency(dep)? {
            metadata.metadata_mut().add_dependency(dep.clone());
        }
    }

    if package.metadata() != metadata.metadata() {
        metadata.write_file()?;
    }

    Ok(())
}

pub fn add_project_optional_dependencies(
    dependencies: &[String],
    group: &str,
    config: &Config,
    options: &AddOptions,
) -> HuakResult<()> {
    let workspace = config.workspace();
    let package = workspace.current_package()?;
    let mut metadata = workspace.current_local_metadata()?;

    // Collect all dependencies that need to be added.
    let mut deps = dependency_iter(dependencies)
        .filter(|dep| {
            !metadata
                .metadata()
                .contains_optional_dependency(dep, group)
                .unwrap_or_default()
        })
        .collect::<Vec<Dependency>>();

    if deps.is_empty() {
        return Ok(());
    };

    let python_env = workspace.resolve_python_environment()?;
    python_env.install_packages(&deps, &options.install_options, config)?;

    // If there's no version data then get the installed version and add to metadata file.
    let packages = python_env.installed_packages()?; // TODO: Only run if versions weren't provided.
    for dep in deps.iter_mut() {
        if dep.requirement().version_or_url.is_none() {
            // TODO: Optimize this .find
            if let Some(pkg) = packages.iter().find(|p| p.name() == dep.name())
            {
                dep.requirement_mut().version_or_url =
                    Some(VersionOrUrl::VersionSpecifier(
                        VersionSpecifiers::from_str(&format!(
                            "=={}",
                            pkg.version()
                        ))
                        .expect("package should have a version"),
                    ));
            }
        }

        if !metadata
            .metadata()
            .contains_optional_dependency(dep, group)?
        {
            metadata
                .metadata_mut()
                .add_optional_dependency(dep.clone(), group);
        }
    }

    if package.metadata() != metadata.metadata() {
        metadata.write_file()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fs, ops::test_config, test_resources_dir_path, Verbosity};
    use tempfile::tempdir;

    #[test]
    fn test_add_project_dependencies() {
        let dir = tempdir().unwrap();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.path().join("mock-project"),
        )
        .unwrap();
        let root = dir.path().join("mock-project");
        let cwd = root.to_path_buf();
        let config = test_config(&root, &cwd, Verbosity::Quiet);
        let ws = config.workspace();
        let venv = ws.resolve_python_environment().unwrap();
        let options = AddOptions {
            install_options: InstallOptions { values: None },
        };

        add_project_dependencies(&[String::from("ruff")], &config, &options)
            .unwrap();

        let dep = Dependency::from_str("ruff").unwrap();
        let metadata = ws.current_local_metadata().unwrap();

        assert!(venv.contains_module("ruff").unwrap());
        assert!(metadata.metadata().contains_dependency(&dep).unwrap());
    }

    #[test]
    fn test_add_optional_project_dependencies() {
        let dir = tempdir().unwrap();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.path().join("mock-project"),
        )
        .unwrap();
        let group = "dev";
        let root = dir.path().join("mock-project");
        let cwd = root.to_path_buf();
        let config = test_config(&root, &cwd, Verbosity::Quiet);
        let ws = config.workspace();
        let venv = ws.resolve_python_environment().unwrap();
        let options = AddOptions {
            install_options: InstallOptions { values: None },
        };

        add_project_optional_dependencies(
            &[String::from("ruff")],
            group,
            &config,
            &options,
        )
        .unwrap();

        let dep = Dependency::from_str("ruff").unwrap();
        let metadata = ws.current_local_metadata().unwrap();

        assert!(venv.contains_module("ruff").unwrap());
        assert!(metadata
            .metadata()
            .contains_optional_dependency(&dep, "dev")
            .unwrap());
    }
}
