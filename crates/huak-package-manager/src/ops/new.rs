use toml_edit::{Item, Table};

use super::{create_workspace, init_git};
use crate::{
    default_package_test_file_contents, importable_package_name, last_path_component, Config,
    Dependency, Error, HuakResult, LocalManifest, WorkspaceOptions,
};
use std::str::FromStr;

pub fn new_app_project(config: &Config, options: &WorkspaceOptions) -> HuakResult<()> {
    new_lib_project(config, options)?;

    let workspace = config.workspace();
    let mut manifest = workspace.current_local_manifest()?;

    let name = last_path_component(workspace.root().as_path())?;
    let as_dep = Dependency::from_str(&name)?;
    manifest.manifest_data_mut().set_project_name(&name);

    let src_path = workspace.root().join("src");
    let importable_name = importable_package_name(as_dep.name())?;
    std::fs::write(
        src_path.join(importable_name).join("main.py"),
        super::DEFAULT_PYTHON_MAIN_FILE_CONTENTS,
    )?;

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

pub fn new_lib_project(config: &Config, options: &WorkspaceOptions) -> HuakResult<()> {
    let workspace = config.workspace();

    // Create a new manifest file or error if one exists.
    let mut manifest = match workspace.current_local_manifest() {
        Ok(_) => return Err(Error::ProjectFound),
        Err(_) => LocalManifest::template(workspace.root().join("pyproject.toml")),
    };

    create_workspace(workspace.root())?;

    if options.uses_git {
        init_git(workspace.root())?;
    }

    let name = &last_path_component(&config.workspace_root)?;
    manifest.manifest_data_mut().set_project_name(name);

    manifest.manifest_data_mut().formatted();
    manifest.write_file()?;
    manifest.write_file()?;

    let as_dep = Dependency::from_str(name)?;
    let src_path = config.workspace_root.join("src");
    let importable_name = importable_package_name(as_dep.name())?;
    std::fs::create_dir_all(src_path.join(&importable_name))?;
    std::fs::create_dir_all(config.workspace_root.join("tests"))?;
    std::fs::write(
        src_path.join(&importable_name).join("__init__.py"),
        super::DEFAULT_PYTHON_INIT_FILE_CONTENTS,
    )?;
    std::fs::write(
        config.workspace_root.join("tests").join("test_version.py"),
        default_package_test_file_contents(&importable_name),
    )
    .map_err(Error::IOError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TerminalOptions, Verbosity};
    use huak_pyproject_toml::value_to_sanitized_string;
    use tempfile::tempdir;

    #[test]
    fn test_new_lib_project() {
        let dir = tempdir().unwrap();
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

        new_lib_project(&config, &options).unwrap();

        let ws = config.workspace();
        let manifest = ws.current_local_manifest().unwrap();
        let test_file_filepath = ws.root().join("tests").join("test_version.py");
        let test_file = std::fs::read_to_string(test_file_filepath).unwrap();
        let expected_test_file = r"from mock_project import __version__


def test_version():
    assert isinstance(__version__, str)
";
        let init_file_filepath = ws
            .root()
            .join("src")
            .join("mock_project")
            .join("__init__.py");
        let init_file = std::fs::read_to_string(init_file_filepath).unwrap();
        let expected_init_file = "__version__ = \"0.0.1\"
";

        assert!(manifest
            .manifest_data()
            .project_table()
            .and_then(|it| it.get("scripts"))
            .is_none());
        assert_eq!(test_file, expected_test_file);
        assert_eq!(init_file, expected_init_file);
    }

    #[test]
    fn test_new_app_project() {
        let dir = tempdir().unwrap();
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

        new_app_project(&config, &options).unwrap();

        let ws = config.workspace();
        let manifest = ws.current_local_manifest().unwrap();
        let main_file_filepath = ws.root().join("src").join("mock_project").join("main.py");
        let main_file = std::fs::read_to_string(main_file_filepath).unwrap();
        let expected_main_file = r#"def main():
    print("Hello, World!")


if __name__ == "__main__":
    main()
"#;

        assert_eq!(
            value_to_sanitized_string(
                manifest
                    .manifest_data()
                    .project_table()
                    .unwrap()
                    .get("scripts")
                    .unwrap()["mock-project"]
                    .as_value()
                    .unwrap()
            ),
            "mock_project.main:main".to_string()
        );
        assert_eq!(main_file, expected_main_file);
    }
}
