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
mod toolchain;
mod update;
mod version;

use crate::{
    default_python_gitignore, env_path_values, git_init, Error, HuakResult, PythonEnvironment,
};
pub use activate::activate_python_environment;
pub use add::{add_project_dependencies, add_project_optional_dependencies, AddOptions};
pub use build::{build_project, BuildOptions};
pub use clean::{clean_project, CleanOptions};
pub use format::{format_project, FormatOptions};
pub use init::{init_app_project, init_lib_project, init_python_env};
pub use install::install;
pub use lint::{lint_project, LintOptions};
pub use new::{new_app_project, new_lib_project};
pub use publish::{publish_project, PublishOptions};
pub use python::{install_python, list_python, use_python};
pub use remove::{remove_project_dependencies, RemoveOptions};
pub use run::run_command_str;
use std::{path::PathBuf, process::Command};
pub use test::{test_project, TestOptions};
pub use toolchain::{
    add_tool, install_toolchain, list_toolchains, remove_tool, run_tool, toolchain_info,
    uninstall_toolchain, update_toolchain, use_toolchain,
};
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
fn add_venv_to_command(cmd: &mut Command, venv: &PythonEnvironment) -> HuakResult<()> {
    let mut paths = env_path_values().unwrap_or_default();

    paths.insert(0, venv.executables_dir_path().clone());
    cmd.env(
        "PATH",
        std::env::join_paths(paths).map_err(|e| Error::InternalError(e.to_string()))?,
    )
    .env("VIRTUAL_ENV", venv.root());

    Ok(())
}

/// Create a workspace directory on the system.
fn create_workspace<T: Into<PathBuf>>(path: T) -> HuakResult<()> {
    let root = path.into();

    if root.exists() {
        return Err(Error::DirectoryExists(root));
    }

    std::fs::create_dir(root)?;

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
