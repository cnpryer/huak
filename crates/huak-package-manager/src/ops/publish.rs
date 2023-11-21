use super::add_venv_to_command;
use crate::{Config, Dependency, HuakResult, InstallOptions};
use std::{process::Command, str::FromStr};

pub struct PublishOptions {
    /// A values vector of publish options typically used for passing on arguments.
    pub values: Option<Vec<String>>,
    pub install_options: InstallOptions,
}

pub fn publish_project(config: &Config, options: &PublishOptions) -> HuakResult<()> {
    let workspace = config.workspace();
    let mut manifest = workspace.current_local_manifest()?;
    let python_env = workspace.resolve_python_environment()?;

    // Install `twine` if it isn't already installed.
    let pub_dep = Dependency::from_str("twine")?;
    if !python_env.contains_module(pub_dep.name())? {
        python_env.install_packages(&[&pub_dep], &options.install_options, config)?;
    }

    // Add the installed `twine` package to the manifest file if it isn't already there.
    if !manifest
        .manifest_data()
        .contains_project_dependency_any(pub_dep.name())
    {
        for pkg in python_env
            .installed_packages()?
            .iter()
            .filter(|pkg| pkg.name() == pub_dep.name())
        {
            manifest
                .manifest_data_mut()
                .add_project_optional_dependency(&pkg.to_string(), "dev");
        }
    }

    manifest.manifest_data_mut().formatted();
    manifest.write_file()?;

    // Run `twine`.
    let mut cmd = Command::new(python_env.python_path());
    let mut args = vec!["-m", "twine", "upload", "dist/*"];
    if let Some(v) = options.values.as_ref() {
        args.extend(v.iter().map(String::as_str));
    }
    add_venv_to_command(&mut cmd, &python_env)?;
    cmd.args(args).current_dir(workspace.root());
    config.terminal().run_command(&mut cmd)
}
