pub mod dist_info;
pub mod index;
pub mod installer;

use core::fmt;
use std::str::FromStr;

use crate::errors::{HuakError, HuakResult};
use pep440_rs::{
    parse_version_specifiers, Operator, Version, VersionSpecifier,
};

/// Version part characters
const VERSION_OPERATOR_CHARACTERS: [char; 5] = ['=', '~', '!', '>', '<'];

/// A Python package struct that encapsulates a packages name and version in accordance with PEP
/// see <https://peps.python.org/pep-0440/>
/// # Examples
/// ```
/// use huak::package::PythonPackage;
///
/// let python_pkg = PythonPackage::from_str_parts("request", Some(">="), "2.28.1").unwrap();
/// println!("I've got 99 {} but huak ain't one", python_pkg);
/// ```
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct PythonPackage {
    /// The name of the python package, pretty straight forward, why are you reading this?
    pub name: String,
    /// The package's version specifier of the package such as `>1.2.3.
    pub version_specifier: Option<VersionSpecifier>,
}

impl PythonPackage {
    pub fn from_str_parts(
        name: &str,
        operator: Option<&str>,
        version: &str,
    ) -> HuakResult<PythonPackage> {
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

/// Instantiate a PythonPackage struct from a String
/// # Arguments
///
/// * 'pkg_string' - A string slice representing PEP-0440 python package
///
/// # Examples
/// ```
/// use huak::package::PythonPackage;
/// use std::str::FromStr;
///
/// let my_pkg = PythonPackage::from_str("requests==2.28.1");
/// ```
impl FromStr for PythonPackage {
    type Err = HuakError;

    fn from_str(pkg_string: &str) -> HuakResult<PythonPackage> {
        // TODO: Improve the method used to parse the version portion
        // Search for the first character that isn't part of the package's name
        let found = pkg_string
            .chars()
            .enumerate()
            .find(|x| VERSION_OPERATOR_CHARACTERS.contains(&x.1));

        let spec_str = match found {
            Some(it) => &pkg_string[it.0..],
            None => {
                return Ok(PythonPackage {
                    name: pkg_string.to_string(),
                    version_specifier: None,
                });
            }
        };

        // TODO: More than one specifier
        match parse_version_specifiers(spec_str) {
            Ok(some_specs) => match some_specs.first() {
                Some(some_spec) => {
                    let name = match pkg_string.strip_suffix(&spec_str) {
                        Some(it) => it,
                        None => pkg_string,
                    };

                    Ok(PythonPackage {
                        name: name.to_string(),
                        version_specifier: Some(some_spec.clone()),
                    })
                }
                None => Ok(PythonPackage {
                    name: pkg_string.to_string(),
                    version_specifier: None,
                }),
            },
            Err(e) => {
                Err(HuakError::PyPackageInitalizationError(e.to_string()))
            }
        }
    }
}

/// Display a PythonPackage as the name and version when available.
/// Can be used to format PythonPackage as a String
/// # Examples
/// ```
/// use huak::package::PythonPackage;
/// use std::str::FromStr;
///
/// let my_pkg = PythonPackage::from_str("requests==2.28.1").unwrap();
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
fn create_operator_from_str(op: &str) -> HuakResult<Operator> {
    Operator::from_str(op)
        .map_err(|_| HuakError::PyPackageInvalidVersionOperator(op.to_string()))
}

// Build a PEP 440 Version from an str.
// Return a Huak-friendly Result.
fn create_version_from_str(ver: &str) -> HuakResult<Version> {
    Version::from_str(ver)
        .map_err(|_| HuakError::PyPackageInvalidVersion(ver.to_string()))
}

// Build a PEP 440 Version Specifier from an Operator and a Version.
// Returns a Huak-friendly Result.
fn create_version_specifier(
    op: Operator,
    ver: Version,
    star: bool,
) -> HuakResult<VersionSpecifier> {
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
            PythonPackage::from_str_parts(pkg_name, None, pkg_version).unwrap();
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
        let new_pkg_from_string = PythonPackage::from_str(&format!(
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
