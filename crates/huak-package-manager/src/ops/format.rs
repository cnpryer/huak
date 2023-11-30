use crate::{Config, Dependency, Error, HuakResult, InstallOptions, PythonEnvironment};
use std::{process::Command, str::FromStr};

use super::add_venv_to_command;

pub struct FormatOptions {
    /// A values vector of format options typically used for passing on arguments.
    pub values: Option<Vec<String>>,
    pub install_options: InstallOptions,
}

/// Format the current project with ruff.
///
/// If the current environment already has ruff installed we'll use that.
/// If ruff isn't found in the current project's environment we try to use
/// ruff from an intalled toolchain. If a toolchain isn't keyed for the project
/// we key the current project with the latest toolchain.
pub fn format_project(config: &Config, options: &FormatOptions) -> HuakResult<()> {
    let ws = config.workspace();
    // TODO(cnpryer): We can technically do this without a current Python environment if we use toolchains.
    let py_env = match ws.current_python_environment() {
        Ok(it) if it.contains_module("ruff")? => it,
        _ => {
            match ws.resolve_local_toolchain(None) {
                Ok(it) => PythonEnvironment::new(it.root().join(".venv"))?,
                Err(Error::ToolchainNotFound) => {
                    // Create a toolchain and return the Python environment it uses (with ruff installed)
                    todo!()
                }
                Err(e) => return Err(e),
            }
        }
    };

    // Run `ruff` for formatting imports and the rest of the Python code in the workspace.
    // NOTE: This needs to be refactored https://github.com/cnpryer/huak/issues/784, https://github.com/cnpryer/huak/issues/718
    let mut terminal = config.terminal();
    let mut cmd = Command::new(py_env.python_path());
    let mut ruff_cmd = Command::new(py_env.python_path());
    let mut ruff_args = vec!["-m", "ruff", "check", ".", "--select", "I", "--fix"];
    add_venv_to_command(&mut cmd, &py_env)?;
    add_venv_to_command(&mut ruff_cmd, &py_env)?;
    let mut args = vec!["-m", "ruff", "format", "."];
    if let Some(v) = options.values.as_ref() {
        args.extend(v.iter().map(String::as_str));
        if v.contains(&"--check".to_string()) {
            terminal.print_warning(
                    "this check will exit early if imports aren't sorted (see https://github.com/cnpryer/huak/issues/510)",
                )?;
            ruff_args.retain(|item| *item != "--fix");
        }
    }
    ruff_cmd.args(ruff_args).current_dir(ws.root());
    terminal.run_command(&mut ruff_cmd)?;
    cmd.args(args).current_dir(ws.root());
    terminal.run_command(&mut cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{copy_dir, initialize_venv, CopyDirOptions, TerminalOptions, Verbosity};
    use huak_dev::dev_resources_dir;
    use tempfile::tempdir;

    #[test]
    fn test_format_project() {
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
        let fmt_filepath = ws.root().join("src").join("mock_project").join("fmt_me.py");
        let pre_fmt_str = r"
def fn( ):
    pass";
        std::fs::write(&fmt_filepath, pre_fmt_str).unwrap();
        let options = FormatOptions {
            values: None,
            install_options: InstallOptions { values: None },
        };

        format_project(&config, &options).unwrap();

        let post_fmt_str = std::fs::read_to_string(&fmt_filepath).unwrap();

        assert_eq!(
            post_fmt_str,
            r"def fn():
    pass
"
        );
    }
}
