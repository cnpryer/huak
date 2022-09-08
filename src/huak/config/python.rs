use super::requirements::PythonPackage;

pub trait PythonConfig {
    fn dependencies(&self) -> &Vec<PythonPackage>;
}
