use super::requirements::PythonPackage;

pub trait PythonConfig {
    fn dependencies(&self) -> &Vec<PythonPackage>;
    fn dev_dependencies(&self) -> &Vec<PythonPackage>;
}
