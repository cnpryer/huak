use core::fmt;

const DEFAULT_VERSION_OP: &str = "==";

/// A Python package struct that captures a packages name and version
/// see https://peps.python.org/pep-0440/
// At the moment (during the PoC phase) the `PythonPackage` contains a
// private string attribute for Huak to utilize.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PythonPackage {
    string: String, // TODO: More like a view maybe.
    pub name: String,
    pub op: Option<String>, // Enum?
    pub version: Option<String>,
}


impl PythonPackage {
    pub fn new(
        name: &str,
        op: Option<&str>,
        version: Option<&str>,
    ) -> PythonPackage {
        let string = package_string_from_parts(name, &op, &version);

        PythonPackage {
            string,
            name: name.to_string(),
            op: op.map(|it| it.to_string()),
            version: version.map(|it| it.to_string()),
        }
    }

    pub fn from(string: String) -> PythonPackage {
        PythonPackage {
            string,
            name: "".to_string(), // TODO
            op: None,
            version: None,
        }
    }

    pub fn string(&self) -> &String {
        &self.string
    }
}

impl fmt::Display for PythonPackage{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // check if a version is specified
        if let Some(ver) = &self.version {
            // check if a version specifier (operator) is supplied
            if let Some(operator) = &self.op {
                write!(f, "{}{}{}", self.name, operator, ver)
            } else {
                // if no version specifier, default to '=='
                write!(f, "{}=={}", self.name, ver)
            }
        } else {
            // if no version, just display python package name
            write!(f, "{}", self.name )
        }
    }

}


fn _package_from_string(
    _string: String,
) -> (String, Option<String>, Option<String>) {
    ("".to_string(), None, None)
}

fn package_string_from_parts(
    name: &str,
    op: &Option<&str>,
    version: &Option<&str>,
) -> String {
    let mut string = name.to_string();

    // If a version was provided but a op was not, then default to '=='.
    let _op = match op {
        Some(it) => it,
        _ => DEFAULT_VERSION_OP,
    };

    if let Some(it) = version {
        string.push_str(_op);
        string.push_str(it);
    };

    string
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_from_parts() {
        let (name1, op1, version1, ans1) =
            ("test", "==", "0.0.0", "test==0.0.0");
        let (name2, version2, ans2) = ("test", "0.0.0", "test==0.0.0");
        let op2: Option<&str> = None;

        let res1 =
            package_string_from_parts(name1, &Some(op1), &Some(version1));
        let res2 = package_string_from_parts(name2, &op2, &Some(version2));

        assert_eq!(res1, ans1);
        assert_eq!(res2, ans2);
    }

    #[test]
    fn python_package_from_new() {
        let pkg_name = "test";
        let pkg_version: Option<&str> = Some("0.0.1");
        let python_pkg = PythonPackage::new(pkg_name, None, pkg_version);
        let test_output = format!("{}", python_pkg);
        assert_eq!(test_output, "test==0.0.1");
    }
}
