use super::{create_workspace, init_git};
use crate::{
    default_package_entrypoint_string, default_package_test_file_contents, importable_package_name,
    last_path_component, Config, Dependency, Error, HuakResult, LocalMetadata, WorkspaceOptions,
};
use std::str::FromStr;

pub fn new_app_project(config: &Config, options: &WorkspaceOptions) -> HuakResult<()> {
    new_lib_project(config, options)?;

    let workspace = config.workspace();
    let mut metadata = workspace.current_local_metadata()?;

    let name = last_path_component(workspace.root().as_path())?;
    let as_dep = Dependency::from_str(&name)?;
    metadata.metadata_mut().set_project_name(name);

    let src_path = workspace.root().join("src");
    let importable_name = importable_package_name(as_dep.name())?;
    std::fs::write(
        src_path.join(&importable_name).join("main.py"),
        super::DEFAULT_PYTHON_MAIN_FILE_CONTENTS,
    )?;
    let entry_point = default_package_entrypoint_string(&importable_name);
    metadata
        .metadata_mut()
        .add_script(as_dep.name(), &entry_point);

    metadata.write_file()
}

pub fn new_lib_project(config: &Config, options: &WorkspaceOptions) -> HuakResult<()> {
    let workspace = config.workspace();

    // Create a new metadata file or error if one exists.
    let mut metadata = match workspace.current_local_metadata() {
        Ok(_) => return Err(Error::ProjectFound),
        Err(_) => LocalMetadata::template(workspace.root().join("pyproject.toml")),
    };

    create_workspace(workspace.root())?;

    if options.uses_git {
        init_git(workspace.root())?;
    }

    let name = &last_path_component(&config.workspace_root)?;
    metadata.metadata_mut().set_project_name(name.to_string());
    metadata.write_file()?;

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
        };
        let options = WorkspaceOptions { uses_git: false };

        new_lib_project(&config, &options).unwrap();

        let ws = config.workspace();
        let metadata = ws.current_local_metadata().unwrap();
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

        assert!(metadata.metadata().project().scripts.is_none());
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
        };
        let options = WorkspaceOptions { uses_git: false };

        new_app_project(&config, &options).unwrap();

        let ws = config.workspace();
        let metadata = ws.current_local_metadata().unwrap();
        let main_file_filepath = ws.root().join("src").join("mock_project").join("main.py");
        let main_file = std::fs::read_to_string(main_file_filepath).unwrap();
        let expected_main_file = r#"def main():
    print("Hello, World!")


if __name__ == "__main__":
    main()
"#;

        assert_eq!(
            metadata.metadata().project().scripts.as_ref().unwrap()["mock-project"],
            format!("{}.main:main", "mock_project")
        );
        assert_eq!(main_file, expected_main_file);
    }
}
