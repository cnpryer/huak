/// A Python package struct.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PythonPackage {
    pub name: String,
    pub version: String,
}
