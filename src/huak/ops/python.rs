use crate::{environment::Environment, Config, Error, HuakResult};
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

    // Get a path to an interpreter based on the version provided.
    let path = match interpreters
        .interpreters()
        .iter()
        .find(|py| py.version().to_string() == version)
        .map(|py| py.path())
    {
        Some(it) => it,
        None => return Err(Error::PythonNotFound),
    };

    // Remove the current Python environment if one exists.
    let workspace = config.workspace();
    match workspace.current_python_environment() {
        Ok(it) => std::fs::remove_dir_all(it.root())?,
        Err(Error::PythonEnvironmentNotFound) => (),
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
    use crate::{ops::test_config, Verbosity};
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
