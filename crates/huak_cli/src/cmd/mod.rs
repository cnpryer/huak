mod activate;
mod add;
mod build;
mod clean;
mod format;
mod init;
mod install;
mod lint;
mod new;
mod publish;
mod python;
mod remove;
mod run;
mod test;
mod update;
mod version;

pub use activate::activate_python_environment;
pub use add::{
    add_project_dependencies, add_project_optional_dependencies, AddOptions,
};
pub use build::{build_project, BuildOptions};
pub use clean::{clean_project, CleanOptions};
pub use format::{format_project, FormatOptions};
use huak_ops::{
    default_python_gitignore, env_path_values, git_init, Error, HuakResult,
    PythonEnvironment,
};
#[allow(unused_imports)]
use huak_ops::{Config, TerminalOptions, Verbosity, Workspace};
pub use init::{init_app_project, init_lib_project};
pub use install::install_project_dependencies;
pub use lint::{lint_project, LintOptions};
pub use new::{new_app_project, new_lib_project};
pub use publish::{publish_project, PublishOptions};
pub use python::{list_python, use_python};
pub use remove::{remove_project_dependencies, RemoveOptions};
pub use run::run_command_str;
use std::{path::PathBuf, process::Command};
pub use test::{test_project, TestOptions};
pub use update::{update_project_dependencies, UpdateOptions};
pub use version::display_project_version;

const DEFAULT_PYTHON_INIT_FILE_CONTENTS: &str = r#"__version__ = "0.0.1"
"#;
const DEFAULT_PYTHON_MAIN_FILE_CONTENTS: &str = r#"def main():
    print("Hello, World!")


if __name__ == "__main__":
    main()
"#;

/// Make a `process::Command` a command with *virtual environment context*.
///
/// - Adds the virtual environment's executables directory path to the top of the command's
///   `PATH` environment variable.
/// - Adds `VIRTUAL_ENV` environment variable to the command pointing at the virtual environment's
///   root.
fn make_venv_command(
    cmd: &mut Command,
    venv: &PythonEnvironment,
) -> HuakResult<()> {
    let mut paths = env_path_values().unwrap_or(Vec::new());

    paths.insert(0, venv.executables_dir_path().clone());
    cmd.env(
        "PATH",
        std::env::join_paths(paths)
            .map_err(|e| Error::InternalError(e.to_string()))?,
    )
    .env("VIRTUAL_ENV", venv.root());

    Ok(())
}

/// Create a workspace directory on the system.
fn create_workspace<T: Into<PathBuf>>(path: T) -> HuakResult<()> {
    let root = path.into();

    if !root.exists() {
        std::fs::create_dir(root)?;
    } else {
        return Err(Error::DirectoryExists(root));
    }

    Ok(())
}

/// Initialize a directory for git.
///
/// - Initializes git
/// - Adds .gitignore if one doesn't already exist.
fn init_git<T: Into<PathBuf>>(path: T) -> HuakResult<()> {
    let root = path.into();

    if !root.join(".git").exists() {
        git_init(&root)?;
    }
    let gitignore_path = root.join(".gitignore");
    if !gitignore_path.exists() {
        std::fs::write(gitignore_path, default_python_gitignore())?;
    }

    Ok(())
}

#[cfg(test)]
pub(crate) mod test_fixtures {
    use super::*;

    /// The resource directory found in the Huak repo used for testing purposes.
    pub(crate) fn test_resources_dir_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("dev-resources")
    }

    pub(crate) fn test_config<T: Into<PathBuf>>(
        root: T,
        cwd: T,
        verbosity: Verbosity,
    ) -> Config {
        let config = Config {
            workspace_root: root.into(),
            cwd: cwd.into(),
            terminal_options: TerminalOptions {
                verbosity,
                ..Default::default()
            },
        };

        config
    }

    pub(crate) fn test_venv(ws: &Workspace) {
        let env = ws.environment();
        let venv_path = format!("{}", ws.root().join(".venv").display());
        let python_path = env.interpreters().latest().unwrap().path();
        let mut cmd = Command::new(python_path);
        cmd.args(["-m", "venv", &venv_path]);
    }
}
