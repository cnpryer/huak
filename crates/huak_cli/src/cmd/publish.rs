use super::make_venv_command;
use huak_ops::{Config, Dependency, HuakResult, InstallOptions};
use std::{process::Command, str::FromStr};

pub struct PublishOptions {
    /// A values vector of publish options typically used for passing on arguments.
    pub values: Option<Vec<String>>,
    pub install_options: InstallOptions,
}

pub fn publish_project(
    config: &Config,
    options: &PublishOptions,
) -> HuakResult<()> {
    let workspace = config.workspace();
    let package = workspace.current_package()?;
    let mut metadata = workspace.current_local_metadata()?;
    let python_env = workspace.resolve_python_environment()?;

    // Install `twine` if it isn't already installed.
    let pub_dep = Dependency::from_str("twine")?;
    if !python_env.contains_module(pub_dep.name())? {
        python_env.install_packages(
            &[&pub_dep],
            &options.install_options,
            config,
        )?;
    }

    // Add the installed `twine` package to the metadata file if it isn't already there.
    if !metadata.metadata().contains_dependency_any(&pub_dep)? {
        for pkg in python_env
            .installed_packages()?
            .iter()
            .filter(|pkg| pkg.name() == pub_dep.name())
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

    // Run `twine`.
    let mut cmd = Command::new(python_env.python_path());
    let mut args = vec!["-m", "twine", "upload", "dist/*"];
    if let Some(v) = options.values.as_ref() {
        args.extend(v.iter().map(|item| item.as_str()));
    }
    make_venv_command(&mut cmd, &python_env)?;
    cmd.args(args).current_dir(workspace.root());
    config.terminal().run_command(&mut cmd)
}
