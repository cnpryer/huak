use std::{ffi::OsString, path::PathBuf};

use crate::python_environment::{
    parse_python_version_from_command, python_paths, Interpreter, Interpreters,
};

/// The `Environment` is a snapshot of the environment.
///
/// `Environment`s would be used for resolving environment variables, the the paths to
/// Python `Interpreters`, etc.
///
/// ```
/// use huak_python_package_manager::Environment;
///
/// let env = Environment::new();
/// let interpreters = env.resolve_interpreters();
/// let python_path = interpreters.latest();
/// ```
pub struct Environment {
    /// Python `Interpreters` installed on the system.
    interpreters: Interpreters,
}

impl Environment {
    /// Initialize an `Environment`.
    pub fn new() -> Environment {
        let interpreters = Environment::resolve_python_interpreters();

        Environment { interpreters }
    }

    /// Get an `Iterator` over the Python `Interpreter` `PathBuf`s found.
    pub fn python_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.interpreters
            .interpreters()
            .iter()
            .map(|interpreter| interpreter.path())
    }

    /// Resolve `Interpreters` for the `Environment`.
    pub fn resolve_python_interpreters() -> Interpreters {
        // Note that we filter out any interpreters we can't establish a `Version` for.
        let interpreters = python_paths().filter_map(|(version, path)| {
            if let Some(v) = version {
                let interpreter = Interpreter::new(path, v);
                Some(interpreter)
            } else if let Ok(Some(v)) = parse_python_version_from_command(&path) {
                let interpreter = Interpreter::new(path, v);
                Some(interpreter)
            } else {
                None
            }
        });

        Interpreters::new(interpreters)
    }

    /// Get a reference to the environment's resolved Python interpreters.
    pub fn interpreters(&self) -> &Interpreters {
        &self.interpreters
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

/// Get a vector of paths from the system `PATH` environment variable.
pub fn env_path_values() -> Option<Vec<PathBuf>> {
    if let Some(val) = env_path_string() {
        return Some(std::env::split_paths(&val).collect());
    }

    None
}

/// Get the OsString value of the enrionment variable `PATH`.
pub fn env_path_string() -> Option<OsString> {
    std::env::var_os("PATH")
}
