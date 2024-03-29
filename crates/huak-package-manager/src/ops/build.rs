use super::add_venv_to_command;
use crate::{Config, Dependency, HuakResult, InstallOptions};
use std::{process::Command, str::FromStr};

pub struct BuildOptions {
    /// A values vector of build options typically used for passing on arguments.
    pub values: Option<Vec<String>>,
    pub install_options: InstallOptions,
}

pub fn build_project(config: &Config, options: &BuildOptions) -> HuakResult<()> {
    let workspace = config.workspace();
    let mut manifest = workspace.current_local_manifest()?;
    let python_env = workspace.resolve_python_environment()?;

    // Install the `build` package if it isn't already installed.
    let build_dep = Dependency::from_str("build")?;
    if !python_env.contains_module(build_dep.name())? {
        python_env.install_packages(&[&build_dep], &options.install_options, config)?;
    }

    // Add the installed `build` package to the manifest file.
    if !manifest
        .manifest_data()
        .contains_project_dependency_any(build_dep.name())
    {
        for pkg in python_env
            .installed_packages()?
            .iter()
            .filter(|pkg| pkg.name() == build_dep.name())
        {
            manifest
                .manifest_data_mut()
                .add_project_optional_dependency(&pkg.to_string(), "dev");
        }
    }

    manifest.manifest_data_mut().formatted();
    manifest.write_file()?;

    // Run `build`.
    let mut cmd = Command::new(python_env.python_path());
    let mut args = vec!["-m", "build"];
    if let Some(it) = options.values.as_ref() {
        args.extend(it.iter().map(std::string::String::as_str));
    }
    add_venv_to_command(&mut cmd, &python_env)?;
    cmd.args(args).current_dir(workspace.root());

    config.terminal().run_command(&mut cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{copy_dir, initialize_venv, CopyDirOptions, TerminalOptions, Verbosity};
    use huak_dev::dev_resources_dir;
    use tempfile::tempdir;

    #[test]
    fn test_build_project() {
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
        let options = BuildOptions {
            values: None,
            install_options: InstallOptions { values: None },
        };

        build_project(&config, &options).unwrap();
    }
}
