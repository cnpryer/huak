use core::fmt;
use std::str::FromStr;

const DEFAULT_VERSION_OP: &str = "==";

/// A Python package struct that captures a packages name and version
/// see https://peps.python.org/pep-0440/
// At the moment (during the PoC phase) the `PythonPackage` contains a
// private string attribute for Huak to utilize.
// #[derive(Clone, Eq, PartialEq, Debug)]
pub struct PythonPackage {
    // string: String, // TODO: More like a view maybe.
    pub name: String,
    pub op: Option<VersionSpecifier>,
    pub version: Option<String>,
}

/// Python Package version specifiers per PEP-0440
/// https://peps.python.org/pep-0440/#version-specifiers
#[derive(PartialEq, Debug)]
pub enum VersionSpecifier {
    Compatible,         // ~=
    Matching,           // == currently the default
    Exclusion,          // !=
    GreaterIncluding,   // >=
    LesserIncluding,    // <=
    GreaterExcluding,   // <
    LesserExcluding,    // >
    ArbitraryEqual,     // ===
}


impl PythonPackage {
    pub fn new(
        name: &str,
        op: Option<&str>,
        version: Option<&str>,
    ) -> PythonPackage {
        if let Some(operator) = op {
            let op_from_string = VersionSpecifier::from_str(operator).unwrap();
            PythonPackage {
                name: name.to_string(),
                op: Some(op_from_string),
                version: version.map(|it| it.to_string()),
            }
        } else {
            PythonPackage {
                name: name.to_string(),
                op: Some(VersionSpecifier::default()),
                version: version.map(|it| it.to_string()),
            }
        }
    }

    pub fn from(pkg_string: String) -> PythonPackage {
        let version_operators = ["==", "~=", "!=", ">=", "<=", ">", "<", "==="].into_iter();
        let mut op = "==";
        let mut op2: Option<&str> = None;
        for i in version_operators {
            if pkg_string.contains(i) {
                op2 = Some(i);
                op = i;
                break;
            }
        }
        return if pkg_string.contains(op) {
            let pkg_components = pkg_string.split(op);
            let pkg_vec = pkg_components.collect::<Vec<&str>>();
            PythonPackage {
                name: pkg_vec[0].to_string(),
                op: Some(VersionSpecifier::from_str(op).unwrap()),
                version: Some(pkg_vec[1].to_string()),
            }
        } else {
            PythonPackage {
                name: pkg_string,
                op: None,
                version: None,
            }
        }
    }

    pub fn string(&self) -> &String {
        &self.name
    }
}

/// Display a PythonPackage as the name and version when available
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

/// Display VersionSpecifier enum (e.g., via "{}")
impl fmt::Display for VersionSpecifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operator = {
            match self {
                VersionSpecifier::Compatible => "~=",
                VersionSpecifier::Matching => "==",
                VersionSpecifier::Exclusion => "!=",
                VersionSpecifier::ArbitraryEqual => "===",
                VersionSpecifier::LesserIncluding=> "<=",
                VersionSpecifier::LesserExcluding => "<",
                VersionSpecifier::GreaterIncluding => ">=",
                VersionSpecifier::GreaterExcluding => ">",
            }
        };
        write!(f, "{}", operator)
    }
}

/// Convert a string to our VersionSpecifier enum
/// can be used like VersionSpecifier::from_str("==").unwrap())
impl FromStr for VersionSpecifier {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "~=" => Ok(VersionSpecifier::Compatible),
            "==" => Ok(VersionSpecifier::Matching),
            "!=" => Ok(VersionSpecifier::Exclusion),
            "===" => Ok(VersionSpecifier::ArbitraryEqual),
            "<=" => Ok(VersionSpecifier::LesserIncluding),
            "<" => Ok(VersionSpecifier::LesserExcluding),
            ">=" => Ok(VersionSpecifier::GreaterIncluding),
            ">" => Ok(VersionSpecifier::GreaterExcluding),
            _ => Err(())
        }
    }
}

/// The default option for VersionSpecifier enum
impl Default for VersionSpecifier {
    fn default() -> Self {
        VersionSpecifier::Matching
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
    fn display_python_package() {
        let pkg_name = "test";
        let pkg_version: Option<&str> = Some("0.0.1");
        let python_pkg = PythonPackage::new(pkg_name, None, pkg_version);
        let py_pkg_fmt = format!("{}", python_pkg);
        assert_eq!(py_pkg_fmt, "test==0.0.1");
    }

    #[test]
    fn display_version_operator() {
        let test_formatted_output = format!("{}", VersionSpecifier::Compatible);
        assert_eq!("~=", test_formatted_output);
    }

    #[test]
    fn create_python_package_struct_from_string() {
        let dependency = "test".to_string();
        let version: String = "0.1.0".to_string();
        let operator: String = "==".to_string();
        let  new_pkg_from_string= PythonPackage::from(format!("{}{}{}", dependency, operator, version));
        assert_eq!(new_pkg_from_string.name, dependency);
        if let Some(op_from_new_pkg) = new_pkg_from_string.op{
            let op_from_new_pkg= format!("{}", op_from_new_pkg);
            assert_eq!(op_from_new_pkg, operator);
        }
        assert_eq!(new_pkg_from_string.version.unwrap(), version);
        let operator: String = "!=".to_string();
        let second_pkg_from_string= PythonPackage::from(format!("{}{}{}", dependency, operator, version));
        assert!(second_pkg_from_string.op.is_some());
        if let Some(op_from_second) = second_pkg_from_string.op{
            let op_from_second= format!("{}", op_from_second);
            assert_eq!(op_from_second, operator);
        }
    }

    #[test]
    fn enum_version_operator_from_string() {
        // ToDo: test that the implementation of FromStr for VersionSpecifier either
        //  1. returns an enum when one of the PEP defined string is passed
        //  2. returns a error (something that can, eventually, be passed to the CLI)
    }

}
