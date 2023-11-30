use super::add_venv_to_command;
use crate::{Config, Dependency, Error, HuakResult, InstallOptions, PythonEnvironment};
use std::{process::Command, str::FromStr};

pub struct LintOptions {
    /// A values vector of lint options typically used for passing on arguments.
    pub values: Option<Vec<String>>,
    pub include_types: bool,
    pub install_options: InstallOptions,
}

pub fn lint_project(config: &Config, options: &LintOptions) -> HuakResult<()> {
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

    let mut terminal = config.terminal();

    if options.include_types {
        // Install `mypy` if it isn't already installed.
        let mypy_dep = Dependency::from_str("mypy")?;
        if !py_env.contains_module("mypy")? {
            py_env.install_packages(&[&mypy_dep], &options.install_options, config)?;
        }

        // Run `mypy` excluding the workspace's Python environment directory.
        let mut mypy_cmd = Command::new(py_env.python_path());
        add_venv_to_command(&mut mypy_cmd, &py_env)?;
        mypy_cmd
            .args(vec!["-m", "mypy", ".", "--exclude", &py_env.name()?])
            .current_dir(ws.root());
        terminal.run_command(&mut mypy_cmd)?;
    }

    // Run `ruff`.
    let mut cmd = Command::new(py_env.python_path());
    let mut args = vec!["-m", "ruff", "check", "."];
    if let Some(v) = options.values.as_ref() {
        args.extend(v.iter().map(String::as_str));
    }
    add_venv_to_command(&mut cmd, &py_env)?;
    cmd.args(args).current_dir(ws.root());
    terminal.run_command(&mut cmd)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{copy_dir, initialize_venv, CopyDirOptions, TerminalOptions, Verbosity};
    use huak_dev::dev_resources_dir;
    use tempfile::tempdir;

    #[test]
    fn test_lint_project() {
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
        let options = LintOptions {
            values: None,
            include_types: true,
            install_options: InstallOptions { values: None },
        };

        lint_project(&config, &options).unwrap();
    }

    #[test]
    fn test_fix_project() {
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
        let options = LintOptions {
            values: Some(vec![String::from("--fix")]),
            include_types: true,
            install_options: InstallOptions { values: None },
        };
        let lint_fix_filepath = ws.root().join("src").join("mock_project").join("fix_me.py");
        let pre_fix_str = r"
import json # this gets removed(autofixed)


def fn():
    pass
";
        let expected = r"


def fn():
    pass
";
        std::fs::write(&lint_fix_filepath, pre_fix_str).unwrap();

        lint_project(&config, &options).unwrap();

        let post_fix_str = std::fs::read_to_string(&lint_fix_filepath).unwrap();

        assert_eq!(post_fix_str, expected);
    }
}
