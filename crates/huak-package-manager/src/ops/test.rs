use super::add_venv_to_command;
use crate::{Config, Dependency, HuakResult, InstallOptions};
use std::{process::Command, str::FromStr};

pub struct TestOptions {
    /// A values vector of test options typically used for passing on arguments.
    pub values: Option<Vec<String>>,
    pub install_options: InstallOptions,
}

pub fn test_project(config: &Config, options: &TestOptions) -> HuakResult<()> {
    let workspace = config.workspace();
    let mut manifest = workspace.current_local_manifest()?;
    let python_env = workspace.resolve_python_environment()?;

    // Install `pytest` if it isn't already installed.
    let test_dep = Dependency::from_str("pytest")?;
    if !python_env.contains_module(test_dep.name())? {
        python_env.install_packages(&[&test_dep], &options.install_options, config)?;
    }

    // Add the installed `pytest` package to the manifest file if it isn't already there.
    if !manifest
        .manifest_data()
        .contains_project_dependency_any(test_dep.name())
    {
        for pkg in python_env
            .installed_packages()?
            .iter()
            .filter(|pkg| pkg.name() == test_dep.name())
        {
            manifest
                .manifest_data_mut()
                .add_project_optional_dependency(&pkg.to_string(), "dev");
        }
    }

    manifest.manifest_data_mut().formatted();
    manifest.write_file()?;

    // Run `pytest` with the package directory added to the command's `PYTHONPATH`.
    let mut cmd = Command::new(python_env.python_path());
    add_venv_to_command(&mut cmd, &python_env)?;
    let python_path = if workspace.root().join("src").exists() {
        workspace.root().join("src")
    } else {
        workspace.root().clone()
    };
    let mut args = vec!["-m", "pytest"];
    if let Some(v) = options.values.as_ref() {
        args.extend(v.iter().map(String::as_str));
    }
    cmd.args(args)
        .env("PYTHONPATH", python_path)
        .current_dir(&config.cwd);
    config.terminal().run_command(&mut cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{copy_dir, initialize_venv, CopyDirOptions, TerminalOptions, Verbosity};
    use huak_dev::dev_resources_dir;
    use tempfile::tempdir;

    #[test]
    fn test_test_project() {
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
        let options = TestOptions {
            values: None,
            install_options: InstallOptions { values: None },
        };

        test_project(&config, &options).unwrap();
    }
}
