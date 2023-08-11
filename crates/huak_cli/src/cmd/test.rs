use super::make_venv_command;
use huak_ops::{Config, Dependency, HuakResult, InstallOptions};
use std::{process::Command, str::FromStr};

pub struct TestOptions {
    /// A values vector of test options typically used for passing on arguments.
    pub values: Option<Vec<String>>,
    pub install_options: InstallOptions,
}

pub fn test_project(config: &Config, options: &TestOptions) -> HuakResult<()> {
    let workspace = config.workspace();
    let package = workspace.current_package()?;
    let mut metadata = workspace.current_local_metadata()?;
    let python_env = workspace.resolve_python_environment()?;

    // Install `pytest` if it isn't already installed.
    let test_dep = Dependency::from_str("pytest")?;
    if !python_env.contains_module(test_dep.name())? {
        python_env.install_packages(
            &[&test_dep],
            &options.install_options,
            config,
        )?;
    }

    // Add the installed `pytest` package to the metadata file if it isn't already there.
    if !metadata.metadata().contains_dependency_any(&test_dep)? {
        for pkg in python_env
            .installed_packages()?
            .iter()
            .filter(|pkg| pkg.name() == test_dep.name())
        {
            metadata.metadata_mut().add_optional_dependency(
                Dependency::from_str(&pkg.to_string())?,
                "dev",
            );
        }
    }

    if package.metadata() != metadata.metadata() {
        metadata.write_file()?;
    }

    // Run `pytest` with the package directory added to the command's `PYTHONPATH`.
    let mut cmd = Command::new(python_env.python_path());
    make_venv_command(&mut cmd, &python_env)?;
    let python_path = if workspace.root().join("src").exists() {
        workspace.root().join("src")
    } else {
        workspace.root().to_path_buf()
    };
    let mut args = vec!["-m", "pytest"];
    if let Some(v) = options.values.as_ref() {
        args.extend(v.iter().map(|item| item.as_str()));
    }
    cmd.args(args)
        .env("PYTHONPATH", python_path)
        .current_dir(&config.cwd);
    config.terminal().run_command(&mut cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::test_fixtures::{
        test_config, test_resources_dir_path, test_venv,
    };
    use huak_ops::{copy_dir, CopyDirOptions, Verbosity};
    use tempfile::tempdir;

    #[test]
    fn test_test_project() {
        let dir = tempdir().unwrap();
        copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.path().join("mock-project"),
            &CopyDirOptions::default(),
        )
        .unwrap();
        let root = dir.path().join("mock-project");
        let cwd = root.to_path_buf();
        let config = test_config(root, cwd, Verbosity::Quiet);
        let ws = config.workspace();
        test_venv(&ws);
        let options = TestOptions {
            values: None,
            install_options: InstallOptions { values: None },
        };

        test_project(&config, &options).unwrap();
    }
}
