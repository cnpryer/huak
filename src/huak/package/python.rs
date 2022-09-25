const DEFAULT_VERSION_OP: &str = "==";

/// A Python package struct.
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
}
