use super::config::Config;
use crate::env::venv::Venv;

/// Traits for a `PythonProject`.
pub trait PythonProject {
    fn config(&self) -> &Config;
    fn venv(&self) -> &Option<Venv>;
    fn set_venv(&mut self, venv: Venv);
}
