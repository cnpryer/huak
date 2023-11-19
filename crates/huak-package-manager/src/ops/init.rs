use toml_edit::{Item, Table};

use super::init_git;
use crate::{
    default_package_entrypoint_string, importable_package_name, last_path_component, Config,
    Dependency, Error, HuakResult, LocalMetadata, WorkspaceOptions,
};
use std::str::FromStr;

pub fn init_app_project(config: &Config, options: &WorkspaceOptions) -> HuakResult<()> {
    init_lib_project(config, options)?;

    let workspace = config.workspace();
    let mut metadata = workspace.current_local_metadata()?;

    let Some(name) = metadata.metadata().project_name() else {
        return Err(Error::InternalError("missing project name".to_string()));
    };
    let as_dep = Dependency::from_str(&name)?;
    let _entry_point = default_package_entrypoint_string(&importable_package_name(as_dep.name())?);

    if let Some(table) = metadata.metadata_mut().project_table_mut() {
        let scripts = &mut table["scripts"];

        if scripts.is_none() {
            *scripts = Item::Table(Table::new());
        }

        let importable = importable_package_name(&name)?;
        scripts[name] = toml_edit::value(format!("{importable}.main:main"));
    }

    metadata.write_file()
}

pub fn init_lib_project(config: &Config, options: &WorkspaceOptions) -> HuakResult<()> {
    let workspace = config.workspace();

    // Create a metadata file or error if one already exists.
    let mut metadata = match workspace.current_local_metadata() {
        Ok(_) => return Err(Error::MetadataFileFound),
        Err(_) => LocalMetadata::template(workspace.root().join("pyproject.toml")),
    };

    if options.uses_git {
        init_git(&config.workspace_root)?;
    }

    let name = last_path_component(&config.workspace_root)?;
    metadata.metadata_mut().set_project_name(&name);
    metadata.write_file()
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
        let metadata = ws.current_local_metadata().unwrap();

        assert_eq!(
            metadata.metadata().to_string(),
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
        let metadata = ws.current_local_metadata().unwrap();

        assert_eq!(
            metadata.metadata().to_string(),
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
