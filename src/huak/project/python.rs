use crate::{config::Config, env::venv::Venv};

pub trait PythonProject {
    fn config(&self) -> &Config;
    fn venv(&self) -> &Option<Venv>;
    fn set_venv(&mut self, venv: Venv);
}
