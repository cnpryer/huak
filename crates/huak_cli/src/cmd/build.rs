use super::make_venv_command;
use huak_ops::{Config, Dependency, HuakResult, InstallOptions};
use std::{process::Command, str::FromStr};

pub struct BuildOptions {
    /// A values vector of build options typically used for passing on arguments.
    pub values: Option<Vec<String>>,
    pub install_options: InstallOptions,
}

pub fn build_project(
    config: &Config,
    options: &BuildOptions,
) -> HuakResult<()> {
    let workspace = config.workspace();
    let package = workspace.current_package()?;
    let mut metadata = workspace.current_local_metadata()?;
    let python_env = workspace.resolve_python_environment()?;

    // Install the `build` package if it isn't already installed.
    let build_dep = Dependency::from_str("build")?;
    if !python_env.contains_module(build_dep.name())? {
        python_env.install_packages(
            &[&build_dep],
            &options.install_options,
            config,
        )?;
    }

    // Add the installed `build` package to the metadata file.
    if !metadata.metadata().contains_dependency_any(&build_dep)? {
        for pkg in python_env
            .installed_packages()?
            .iter()
            .filter(|pkg| pkg.name() == build_dep.name())
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

    // Run `build`.
    let mut cmd = Command::new(python_env.python_path());
    let mut args = vec!["-m", "build"];
    if let Some(it) = options.values.as_ref() {
        args.extend(it.iter().map(|item| item.as_str()));
    }
    make_venv_command(&mut cmd, &python_env)?;
    cmd.args(args).current_dir(workspace.root());

    config.terminal().run_command(&mut cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        fs::{self, CopyDirOptions},
        sys::Verbosity,
        test::{test_config, test_resources_dir_path, test_venv},
    };
    use tempfile::tempdir;

    #[test]
    fn test_build_project() {
        let dir = tempdir().unwrap();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.path().join("mock-project"),
            &CopyDirOptions::default(),
        )
        .unwrap();
        let root = dir.path().join("mock-project");
        let cwd = dir.path().to_path_buf();
        let config = test_config(root, cwd, Verbosity::Quiet);
        let ws = config.workspace();
        test_venv(&ws);
        let options = BuildOptions {
            values: None,
            install_options: InstallOptions { values: None },
        };

        build_project(&config, &options).unwrap();
    }
}
