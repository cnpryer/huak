use toml_edit::{Item, Table};

use super::init_git;
use crate::{
    default_package_entrypoint_string, directory_is_venv, importable_package_name,
    last_path_component, Config, Dependency, Error, HuakResult, InstallOptions, LocalManifest,
    WorkspaceOptions,
};
use std::{path::PathBuf, str::FromStr};

pub fn init_app_project(config: &Config, options: &WorkspaceOptions) -> HuakResult<()> {
    init_lib_project(config, options)?;

    let workspace = config.workspace();
    let mut manifest = workspace.current_local_manifest()?;

    let Some(name) = manifest.manifest_data().project_name() else {
        return Err(Error::InternalError("missing project name".to_string()));
    };
    let as_dep = Dependency::from_str(&name)?;
    let _entry_point = default_package_entrypoint_string(&importable_package_name(as_dep.name())?);

    if let Some(table) = manifest.manifest_data_mut().project_table_mut() {
        let scripts = &mut table["scripts"];

        if scripts.is_none() {
            *scripts = Item::Table(Table::new());
        }

        let importable = importable_package_name(&name)?;
        scripts[name] = toml_edit::value(format!("{importable}.main:main"));
    }

    manifest.write_file()
}

pub fn init_lib_project(config: &Config, options: &WorkspaceOptions) -> HuakResult<()> {
    let workspace = config.workspace();

    // Create a manifest file or error if one already exists.
    let mut manifest = match workspace.current_local_manifest() {
        Ok(_) => return Err(Error::ManifestFileFound),
        Err(_) => LocalManifest::template(workspace.root().join("pyproject.toml")),
    };

    if options.uses_git {
        init_git(&config.workspace_root)?;
    }

    let name = last_path_component(&config.workspace_root)?;
    manifest.manifest_data_mut().set_project_name(&name);
    manifest.write_file()
}

// TODO(cnpryer): Remove current huak install ops
pub fn init_python_env(
    manifest: Option<PathBuf>,
    optional_dependencies: Option<Vec<String>>,
    force: bool,
    options: &InstallOptions,
    config: &Config,
) -> HuakResult<()> {
    let ws = config.workspace();

    // TODO(cnpryer): Can't remember if clap parses "." as curr dir
    let mut manifest_path = manifest.unwrap_or(ws.root().join("pyproject.toml"));
    if manifest_path
        .file_name()
        .is_some_and(|it| !it.eq_ignore_ascii_case("pyproject.toml"))
    {
        return Err(Error::ManifestFileNotSupported(manifest_path));
    }

    manifest_path.set_file_name("pyproject.toml");

    let Ok(manifest) = LocalManifest::new(manifest_path) else {
        return config
            .terminal()
            .print_warning("a manifest file could not be resolved");
    };

    let mut dependencies = Vec::new();

    if let Some(gs) = optional_dependencies {
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

    // TODO(cnpryer): Relax this by attempting to use existing environments
    if force {
        // Remove the current Python virtual environment if one exists.
        match ws.current_python_environment() {
            Ok(it) if directory_is_venv(it.root()) => std::fs::remove_dir_all(it.root())?,
            // TODO(cnpryer): This might be a clippy bug.
            #[allow(clippy::no_effect)]
            Ok(_)
            | Err(Error::PythonEnvironmentNotFound | Error::UnsupportedPythonEnvironment(_)) => {
                ();
            }
            Err(e) => return Err(e),
        };
    }

    let python_env = ws.resolve_python_environment()?;
    python_env.install_packages(&dependencies, options, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        copy_dir, default_pyproject_toml_contents, initialize_venv, CopyDirOptions, Package,
        TerminalOptions, Verbosity,
    };
    use huak_dev::dev_resources_dir;
    use tempfile::tempdir;

    #[test]
    fn test_init_lib_project() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join("mock-project")).unwrap();
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
        let options = WorkspaceOptions {
            uses_git: false,
            values: None,
        };
        init_lib_project(&config, &options).unwrap();

        let ws = config.workspace();
        let manifest = ws.current_local_manifest().unwrap();

        assert_eq!(
            manifest.manifest_data().to_string(),
            default_pyproject_toml_contents("mock-project")
        );
    }

    #[test]
    fn test_init_app_project() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join("mock-project")).unwrap();
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
        let options = WorkspaceOptions {
            uses_git: false,
            values: None,
        };

        init_app_project(&config, &options).unwrap();

        let ws = config.workspace();
        let manifest = ws.current_local_manifest().unwrap();

        assert_eq!(
            manifest.manifest_data().to_string(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock-project"
version = "0.0.1"
description = ""
dependencies = []

[project.scripts]
mock-project = "mock_project.main:main"
"#
        );
    }

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

        init_python_env(None, None, true, &options, &config).unwrap();

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

        init_python_env(
            None,
            Some(vec![String::from("dev")]),
            true,
            &options,
            &config,
        )
        .unwrap();

        assert!(!had_package);
        assert!(venv.contains_module("pytest").unwrap());
    }
}
