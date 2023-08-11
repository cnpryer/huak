use huak_ops::{
    active_python_env_path, directory_is_venv, venv_executables_dir_path,
    Config, Environment, Error, HuakResult,
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

pub fn use_python(version: &str, config: &Config) -> HuakResult<()> {
    let interpreters = Environment::resolve_python_interpreters();

    // Get a path to an interpreter based on the version provided, excluding any activated Python environment.
    let path = match interpreters
        .interpreters()
        .iter()
        .filter(|py| {
            !active_python_env_path().map_or(false, |it| {
                py.path().parent() == Some(&venv_executables_dir_path(it))
            })
        })
        .find(|py| py.version().to_string() == version)
        .map(|py| py.path())
    {
        Some(it) => it,
        None => return Err(Error::PythonNotFound),
    };

    // Remove the current Python virtual environment if one exists.
    let workspace = config.workspace();
    match workspace.current_python_environment() {
        Ok(it) if directory_is_venv(it.root()) => {
            std::fs::remove_dir_all(it.root())?
        }
        Ok(_) => (),
        Err(Error::PythonEnvironmentNotFound)
        | Err(Error::UnsupportedPythonEnvironment(_)) => (),
        Err(e) => return Err(e),
    };

    // Create a new Python environment using the interpreter matching the version provided.
    let mut cmd = Command::new(path);
    cmd.args(["-m", "venv", ".venv"])
        .current_dir(&config.workspace_root);
    config.terminal().run_command(&mut cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::test_fixtures::test_config;
    use huak_ops::Verbosity;
    use tempfile::tempdir;

    #[test]
    fn test_use_python() {
        let dir = tempdir().unwrap();
        let interpreters = Environment::resolve_python_interpreters();
        let version = interpreters.latest().unwrap().version();
        let root = dir.path();
        let cwd = root;
        let config = test_config(root, cwd, Verbosity::Quiet);

        use_python(&version.to_string(), &config).unwrap();
    }
}
