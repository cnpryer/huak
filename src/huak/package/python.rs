use core::fmt;
use std::str::FromStr;

use crate::errors::HuakError;

/// Version operators used in dependency strings.
const VERSION_OPERATORS: [&str; 8] =
    ["==", "~=", "!=", ">=", "<=", ">", "<", "==="];

/// A Python package struct that encapsulates a packages name and version in accordance with PEP
/// see <https://peps.python.org/pep-0440/>
/// # Examples
/// ```
/// use huak::package::python::PythonPackage;
/// let python_pkg = PythonPackage::new("request", Some(">="), Some("2.28.1")).unwrap();
/// // or
/// let other_pkg = PythonPackage::from("problems==0.0.2").unwrap();
/// println!("I've got 99 {} but huak ain't one", other_pkg);
/// ```
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PythonPackage {
    /// The name of the python package, pretty straight forward, why are you reading this?
    pub name: String,
    /// Th operator represents PEP's Version Specifiers, such as "==" or "<="
    pub operator: Option<VersionOp>,
    /// The semantic version associated with a python package
    pub version: Option<String>,
}

/// Python Package version specifiers per PEP-0440
/// <https://peps.python.org/pep-0440/#version-specifiers>
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum VersionOp {
    /// "~=" Releases expected to be compatible with this version (e.g. ~= 1.4)
    Compatible,
    /// "==" Releases matching this version (e.g., == 1.4.1) the default
    Matching,
    /// "!=" Version Exclusion (e.g., != 1.4*)
    Exclusion,
    /// ">=" Releases greater than or equal to this version
    GreaterIncluding,
    /// "<=" Releases lesser than or equal to this version
    LesserIncluding,
    /// "<" Releases greater than this version
    GreaterExcluding,
    /// ">" Releases lesser than this version
    LesserExcluding,
    /// "===" Releases that do not match semantic version (e.g., ===foobar)
    /// maintained for legacy purposes, do not use unless necessary
    ArbitraryEqual,
}

impl PythonPackage {
    pub fn new(
        name: &str,
        operator: Option<&str>,
        version: Option<&str>,
    ) -> Result<PythonPackage, HuakError> {
        let op = match operator {
            Some(it) => Some(VersionOp::from_str(it)?),
            None => {
                if version.is_none() {
                    None
                } else {
                    Some(VersionOp::default())
                }
            }
        };

        let ver = version.map(|it| it.to_string());

        Ok(PythonPackage {
            name: name.to_string(),
            operator: op,
            version: ver,
        })
    }

    /// Instantiate a PythonPackage struct from a String
    /// # Arguments
    ///
    /// * 'pkg_string' - A string slice representing PEP-0440 python package
    ///
    /// # Examples
    /// ```
    /// use huak::package::python::PythonPackage;
    /// let my_pkg = PythonPackage::from("requests==2.28.1");
    /// ```
    pub fn from(pkg_string: &str) -> Result<PythonPackage, HuakError> {
        // unfortunately, we have to redeclare the operators here or bring in a 3rd party crate (like strum)
        // to derive an iterable from out VersionOp enum
        let version_operators = VERSION_OPERATORS.into_iter();
        let mut op: Option<&str> = None;
        // TODO: Collect from filter on iter. Maybe contains.
        for i in version_operators {
            if pkg_string.contains(i) {
                op = Some(i);
                break;
            }
        }
        let package = match op {
            Some(it) => {
                let pkg_components = pkg_string.split(it);
                let pkg_vec = pkg_components.collect::<Vec<&str>>();
                PythonPackage {
                    name: pkg_vec[0].to_string(),
                    operator: Some(VersionOp::from_str(it)?),
                    version: Some(pkg_vec[1].to_string()),
                }
            }
            None => PythonPackage {
                name: pkg_string.to_string(),
                operator: None,
                version: None,
            },
        };

        Ok(package)
    }

    pub fn string(&self) -> &String {
        &self.name
    }
}

/// Display a PythonPackage as the name and version when available.
/// Can be used to format PythonPackage as a String
/// # Examples
/// ```
/// use huak::package::python::PythonPackage;
/// let my_pkg = PythonPackage::from("requests==2.28.1").unwrap();
/// println!("{}", my_pkg); // output: "request==2.28.1"
/// ```
impl fmt::Display for PythonPackage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // check if a version is specified
        if let Some(ver) = &self.version {
            // check if a version specifier (operator) is supplied
            if let Some(operator) = &self.operator {
                write!(f, "{}{}{}", self.name, operator, ver)
            } else {
                // if no version specifier, default to '=='
                write!(f, "{}=={}", self.name, ver)
            }
        } else {
            // if no version, just display python package name
            write!(f, "{}", self.name)
        }
    }
}

/// Display VersionOp enum (e.g., format!("{}", my_version_op);)
/// used internally by PythonPackage
/// # Examples
/// ```
/// use huak::package::python::VersionOp;
/// println!("{}", VersionOp::Exclusion); // output: "!="
/// let ver_string = format!("{}", VersionOp::Compatible); // ver_string is "~="
/// ```
impl fmt::Display for VersionOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operator = {
            match self {
                VersionOp::Compatible => "~=",
                VersionOp::Matching => "==",
                VersionOp::Exclusion => "!=",
                VersionOp::ArbitraryEqual => "===",
                VersionOp::LesserIncluding => "<=",
                VersionOp::LesserExcluding => "<",
                VersionOp::GreaterIncluding => ">=",
                VersionOp::GreaterExcluding => ">",
            }
        };
        write!(f, "{}", operator)
    }
}

/// Convert a string to our VersionOp enum
/// used internally by PythonPackage
///
/// # Examples
/// ```
/// use std::str::FromStr;
/// use huak::package::python::VersionOp;
/// let ver_op_string = "==";
/// let ver_op_enum = VersionOp::from_str(ver_op_string).unwrap();
/// ```
impl FromStr for VersionOp {
    type Err = HuakError;
    fn from_str(s: &str) -> Result<Self, self::HuakError> {
        match s {
            "~=" => Ok(VersionOp::Compatible),
            "==" => Ok(VersionOp::Matching),
            "!=" => Ok(VersionOp::Exclusion),
            "===" => Ok(VersionOp::ArbitraryEqual),
            "<=" => Ok(VersionOp::LesserIncluding),
            "<" => Ok(VersionOp::LesserExcluding),
            ">=" => Ok(VersionOp::GreaterIncluding),
            ">" => Ok(VersionOp::GreaterExcluding),
            _ => Err(self::HuakError::InvalidPyPackageVersionOp(s.to_string())),
        }
    }
}

/// Set the VersionOp enum to the default value
impl Default for VersionOp {
    fn default() -> Self {
        VersionOp::Matching
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test future changes do not break PythonPackage implementation std::fmt::Display
    #[test]
    fn display_python_package() {
        let pkg_name = "test";
        let pkg_version: Option<&str> = Some("0.0.1");
        let python_pkg =
            PythonPackage::new(pkg_name, None, pkg_version).unwrap();
        let py_pkg_fmt = format!("{}", python_pkg);
        assert_eq!(py_pkg_fmt, "test==0.0.1");
    }
    // test future changes do not break VersionOp's implementation std::fmt::Display
    #[test]
    fn display_version_operator() {
        let compatible_fmt = format!("{}", VersionOp::Compatible);
        assert_eq!("~=", compatible_fmt);
    }

    // Test that PythonPackage::from creates a new instance from a string
    #[test]
    fn create_python_package_struct_from_string() {
        let dependency = "test".to_string();
        let version: String = "0.1.0".to_string();
        let operator: String = "!=".to_string();
        let new_pkg_from_string = PythonPackage::from(&format!(
            "{}{}{}",
            dependency, operator, version
        ))
        .unwrap();
        assert_eq!(new_pkg_from_string.name, dependency);
        if let Some(op_from_new_pkg) = new_pkg_from_string.operator {
            assert_eq!(format!("{}", op_from_new_pkg), operator);
        }
        assert_eq!(new_pkg_from_string.version.unwrap(), version);
    }

    #[test]
    fn enum_version_operator_from_string() {
        // ToDo: test that the implementation of FromStr for VersionOp either
        //  1. returns an enum when one of the PEP defined string is passed
        //  2. returns a error (something that can, eventually, be passed to the CLI)
        //  the second part in particular is what needs to be implemented and tested
    }

    #[test]
    fn py_pkg_with_multiple_version_specifier() {
        // ToDo: PEP allows multiple version specifiers via a comma seperated list
        //  example: "torch>=1.0.0,!=1.8.0" or in english, greater than or equal to 1.0.0 but not 1.8.0
        //  right now, a PythonPackage only holds 1 version, & 1 version operator
    }
}
