/// A Python package struct.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PythonPackage {
    pub name: String,
    pub version: String,
}

impl PythonPackage {
    pub fn new(name: String) -> PythonPackage {
        PythonPackage {
            name,
            version: "".to_string(),
        }
    }
}
