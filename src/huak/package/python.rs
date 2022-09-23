/// A Python package struct.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PythonPackage {
    string: String,
    pub name: String,
    pub version: String,
}

impl PythonPackage {
    pub fn new(string: String) -> PythonPackage {
        PythonPackage {
            string,
            name: "".to_string(),
            version: "".to_string(),
        }
    }

    pub fn string(&self) -> &String {
        &self.string
    }
}
