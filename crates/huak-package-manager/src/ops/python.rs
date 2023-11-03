use crate::{
    active_python_env_path, directory_is_venv, venv_executables_dir_path, Config, Environment,
    Error, HuakResult,
};
use huak_home::huak_home_dir;
use huak_python_manager::{
    install_with_target, resolve_release, Options, RequestedVersion, Strategy,
};
use std::process::Command;
use termcolor::Color;

pub fn list_python(config: &Config) -> HuakResult<()> {
    let env = Environment::new();

    // Print enumerated Python paths as they exist in the `PATH` environment variable.
    env.python_paths().enumerate().for_each(|(i, path)| {
        config
            .terminal()
            .print_custom(i + 1, path.display(), Color::Blue, false)
            .ok();
    });

    Ok(())
}

pub fn use_python(version: &RequestedVersion, config: &Config) -> HuakResult<()> {
    let interpreters = Environment::resolve_python_interpreters();

    // TODO(cnpryer): Re-export `Interpreter` as public
    // Get a path to an interpreter based on the version provided, excluding any activated Python environment.
    #[allow(clippy::redundant_closure_for_method_calls)]
    let Some(path) = interpreters
        .interpreters()
        .iter()
        .filter(|py| {
            !active_python_env_path().map_or(false, |it| {
                py.path().parent() == Some(&venv_executables_dir_path(it))
            })
        })
        .find(|py| version.matches_version(py.version()))
        .map(|py| py.path())
    else {
        return Err(Error::PythonNotFound);
    };

    // Remove the current Python virtual environment if one exists.
    let workspace = config.workspace();
    match workspace.current_python_environment() {
        Ok(it) if directory_is_venv(it.root()) => std::fs::remove_dir_all(it.root())?,
        // TODO(cnpryer): This might be a clippy bug.
        #[allow(clippy::no_effect)]
        Ok(_) | Err(Error::PythonEnvironmentNotFound | Error::UnsupportedPythonEnvironment(_)) => {
            ();
        }
        Err(e) => return Err(e),
    };

    // Create a new Python environment using the interpreter matching the version provided.
    let mut cmd = Command::new(path);
    cmd.args(["-m", "venv", ".venv"])
        .current_dir(&config.workspace_root);
    config.terminal().run_command(&mut cmd)
}

pub fn install_python(version: &RequestedVersion) -> HuakResult<()> {
    // Use default selection strategy to find the best match for the requested version.
    let strategy = Strategy::Selection(Options {
        version: Some(version.clone()),
        ..Default::default()
    });

    let Some(release) = resolve_release(&strategy) else {
        return Err(Error::PythonReleaseNotFound(version.to_string()));
    };

    // Always install to Huak's toolchain.
    let Some(target) = huak_home_dir().map(|it| {
        it.join("toolchains").join(format!(
            "huak-{}-{}-{}-{}",
            release.kind, release.version, release.os, release.architecture
        ))
    }) else {
        return Err(Error::HuakHomeNotFound);
    };

    install_with_target(&release, target).map_err(|e| Error::PythonInstallError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TerminalOptions, Verbosity};
    use tempfile::tempdir;

    #[test]
    fn test_use_python() {
        let dir = tempdir().unwrap();
        let interpreters = Environment::resolve_python_interpreters();
        let version = interpreters.latest().unwrap().version();
        let version = RequestedVersion {
            major: version.major,
            minor: version.minor,
            patch: None,
        };
        let workspace_root = dir.path().to_path_buf();
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

        use_python(&version, &config).unwrap();
    }
}
