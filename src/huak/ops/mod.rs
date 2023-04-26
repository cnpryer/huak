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

#[allow(unused_imports)]
use crate::{
    config::Config,
    sys::{TerminalOptions, Verbosity},
    workspace::Workspace,
};
use crate::{
    environment::env_path_values, git, python_environment::PythonEnvironment,
    Error, HuakResult,
};
pub use activate::activate_python_environment;
pub use add::{
    add_project_dependencies, add_project_optional_dependencies, AddOptions,
};
pub use build::{build_project, BuildOptions};
pub use clean::{clean_project, CleanOptions};
pub use format::{format_project, FormatOptions};
pub use init::{init_app_project, init_lib_project};
pub use install::install_project_dependencies;
pub use lint::{lint_project, LintOptions};
pub use new::{new_app_project, new_lib_project};
pub use publish::{publish_project, PublishOptions};
pub use python::{list_python, use_python};
pub use remove::{remove_project_dependencies, RemoveOptions};
pub use run::run_command_str;
use std::{path::Path, process::Command};
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
fn create_workspace<T: AsRef<Path>>(path: T) -> HuakResult<()> {
    let root = path.as_ref();

    if !root.exists() {
        std::fs::create_dir(root)?;
    } else {
        return Err(Error::DirectoryExists(root.to_path_buf()));
    }

    Ok(())
}

/// Initialize a directory for git.
///
/// - Initializes git
/// - Adds .gitignore if one doesn't already exist.
fn init_git<T: AsRef<Path>>(path: T) -> HuakResult<()> {
    let root = path.as_ref();

    if !root.join(".git").exists() {
        git::init(root)?;
    }
    let gitignore_path = root.join(".gitignore");
    if !gitignore_path.exists() {
        std::fs::write(gitignore_path, git::default_python_gitignore())?;
    }

    Ok(())
}

#[cfg(test)]
fn test_config<T: AsRef<Path>>(
    root: T,
    cwd: T,
    verbosity: Verbosity,
) -> Config {
    let config = Config {
        workspace_root: root.as_ref().to_path_buf(),
        cwd: cwd.as_ref().to_path_buf(),
        terminal_options: TerminalOptions { verbosity },
    };

    config
}

#[cfg(test)]
fn test_venv(ws: &Workspace) {
    let env = ws.environment();
    let venv_path = format!("{}", ws.root().join(".venv").display());
    let python_path = env.interpreters().latest().unwrap().path();
    let mut cmd = Command::new(python_path);
    cmd.args(["-m", "venv", &venv_path]);
}
