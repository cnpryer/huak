use toml_edit::{Item, Table};

use super::init_git;
use crate::{
    default_package_entrypoint_string, importable_package_name, last_path_component, Config,
    Dependency, Error, HuakResult, LocalManifest, WorkspaceOptions,
};
use std::str::FromStr;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{default_pyproject_toml_contents, TerminalOptions, Verbosity};
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
        let options = WorkspaceOptions { uses_git: false };
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
        let options = WorkspaceOptions { uses_git: false };

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
}
