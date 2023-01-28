pub mod index;

use core::fmt;
use std::str::FromStr;

use crate::errors::HuakError;
use pep440_rs::{Operator, Version, VersionSpecifier};

/// Version operators used in dependency strings.
const VERSION_OPERATORS: [&str; 8] =
    ["==", "~=", "!=", ">=", "<=", ">", "<", "==="];

/// A Python package struct that encapsulates a packages name and version in accordance with PEP
/// see <https://peps.python.org/pep-0440/>
/// # Examples
/// ```
/// use huak::package::PythonPackage;
/// let python_pkg = PythonPackage::new("request", Some(">="), "2.28.1").unwrap();
/// // or
/// let other_pkg = PythonPackage::from("problems==0.0.2").unwrap();
/// println!("I've got 99 {} but huak ain't one", other_pkg);
/// ```
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct PythonPackage {
    /// The name of the python package, pretty straight forward, why are you reading this?
    pub name: String,
    /// The package's version specifier of the package such as `>1.2.3.
    pub version_specifier: Option<VersionSpecifier>,
}

impl PythonPackage {
    pub fn new(
        name: &str,
        operator: Option<&str>,
        version: &str,
    ) -> Result<PythonPackage, HuakError> {
        let op = match operator {
            Some(it) => create_operator_from_str(it)?,
            None => Operator::Equal,
        };
        let ver = create_version_from_str(version)?;
        let specifier = create_version_specifier(op, ver, false)?;

        Ok(PythonPackage {
            name: name.to_string(),
            version_specifier: Some(specifier),
        })
    }

    /// Instantiate a PythonPackage struct from a String
    /// # Arguments
    ///
    /// * 'pkg_string' - A string slice representing PEP-0440 python package
    ///
    /// # Examples
    /// ```
    /// use huak::package::PythonPackage;
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
                let name = pkg_vec[0].to_string();
                let operator = create_operator_from_str(it)?;
                let version = create_version_from_str(pkg_vec[1])?;
                let specifier =
                    create_version_specifier(operator, version, false)?;
                PythonPackage {
                    name,
                    version_specifier: Some(specifier),
                }
            }
            None => PythonPackage {
                name: pkg_string.to_string(),
                ..Default::default()
            },
        };

        Ok(package)
    }

    pub fn string(&self) -> &String {
        &self.name
    }

    pub fn operator(&self) -> Option<&Operator> {
        if let Some(specifier) = &self.version_specifier {
            return Some(specifier.operator());
        }

        None
    }

    pub fn version(&self) -> Option<&Version> {
        if let Some(specifier) = &self.version_specifier {
            return Some(specifier.version());
        }

        None
    }
}

/// Display a PythonPackage as the name and version when available.
/// Can be used to format PythonPackage as a String
/// # Examples
/// ```
/// use huak::package::PythonPackage;
/// let my_pkg = PythonPackage::from("requests==2.28.1").unwrap();
/// println!("{}", my_pkg); // output: "request==2.28.1"
/// ```
impl fmt::Display for PythonPackage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // check if a version is specified
        if let Some(ver) = self.version() {
            // check if a version specifier (operator) is supplied
            if let Some(operator) = self.operator() {
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

// Build a PEP 440 Version Operator from an str.
// Return a Huak-friendly Result.
fn create_operator_from_str(op: &str) -> Result<Operator, HuakError> {
    Operator::from_str(op)
        .map_err(|_| HuakError::PyPackageInvalidOperator(op.to_string()))
}

// Build a PEP 440 Version from an str.
// Return a Huak-friendly Result.
fn create_version_from_str(ver: &str) -> Result<Version, HuakError> {
    Version::from_str(ver)
        .map_err(|_| HuakError::PyPackageInvalidVersion(ver.to_string()))
}

// Build a PEP 440 Version Specifier from an Operator and a Version.
// Returns a Huak-friendly Result.
fn create_version_specifier(
    op: Operator,
    ver: Version,
    star: bool,
) -> Result<VersionSpecifier, HuakError> {
    VersionSpecifier::new(op, ver, star)
        .map_err(|_| HuakError::PyPackageVersionSpecifierError)
}

#[cfg(test)]
mod tests {
    use super::*;

    // test future changes do not break PythonPackage implementation std::fmt::Display
    #[test]
    fn display_python_package() {
        let pkg_name = "test";
        let pkg_version = "0.0.1";
        let python_pkg =
            PythonPackage::new(pkg_name, None, pkg_version).unwrap();
        let py_pkg_fmt = format!("{}", python_pkg);
        assert_eq!(py_pkg_fmt, "test==0.0.1");
    }
    // test future changes do not break VersionOp's implementation std::fmt::Display
    #[test]
    fn display_version_operator() {
        let compatible_fmt = format!("{}", Operator::TildeEqual);
        assert_eq!("~=", compatible_fmt);
    }

    // Test that PythonPackage::from creates a new instance from a string
    #[test]
    fn create_python_package_struct_from_string() {
        let dependency = "test".to_string();
        let version = "0.1.0";
        let operator = "!=";
        let new_pkg_from_string = PythonPackage::from(&format!(
            "{}{}{}",
            dependency, operator, version
        ))
        .unwrap();
        assert_eq!(new_pkg_from_string.name, dependency);
        if let Some(it) = new_pkg_from_string.operator() {
            assert_eq!(format!("{}", it), operator);
        }
        assert_eq!(
            new_pkg_from_string.version().unwrap(),
            &Version::from_str(version).unwrap()
        );
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
