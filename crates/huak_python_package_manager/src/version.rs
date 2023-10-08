use std::{cmp::Ordering, fmt::Display, str::FromStr};

use regex::{Captures, Regex};

use crate::{Error, HuakResult};

/// A trait used to convert a struct to `SemVer`.
trait ToSemVer {
    /// Convert to `SemVer` (MAJOR.MINOR.PATCH).
    fn to_semver(self) -> SemVer;
}

#[derive(Debug)]
/// A generic `Version` struct.
///
/// This struct is mainly used for the Python `Interpreter`.
pub struct Version {
    release: Vec<usize>,
}

impl Version {
    #[must_use]
    pub fn release(&self) -> &Vec<usize> {
        &self.release
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            self.release[0], self.release[1], self.release[2]
        ) // TODO
    }
}

struct SemVer {
    major: usize,
    minor: usize,
    patch: usize,
}

impl ToSemVer for Version {
    fn to_semver(self) -> SemVer {
        SemVer {
            major: self.release[0],
            minor: self.release[1],
            patch: self.release[2],
        }
    }
}

impl Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch) // TODO
    }
}

/// Initialize a `Version` from a `&str`.
///
/// ```
/// use huak_python_package_manager::Version;
///
/// let a = Version::from_str("0.0.1").unwrap();
/// let b = Version::from_str("0.0.2").unwrap();
///
/// assert!(a < b);
/// ```
impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Get potential `Version` parts from a `&str` (N.N.N).
        let captures = captures_version_str(s)?;
        let release = parse_semver_from_captures(&captures)?;

        if release.len() != 3 {
            return Err(Error::InvalidVersionString(format!(
                "{s} must be SemVer-compatible"
            )));
        }

        let version = Version { release };

        Ok(version)
    }
}

/// Use regex to capture potential `Version` numbers from a `&str`.
fn captures_version_str(s: &str) -> HuakResult<Captures> {
    let re = Regex::new(r"^(\d+)(?:\.(\d+))?(?:\.(\d+))?$")?;
    let Some(captures) = re.captures(s) else {
        return Err(Error::InvalidVersionString(s.to_string()));
    };
    Ok(captures)
}

/// A naive parsing of semantic version parts from `Regex::Captures`.
///
/// Expects three parts (MAJOR.MINOR.PATCH) and defaults each part to 0.
fn parse_semver_from_captures(captures: &Captures) -> HuakResult<Vec<usize>> {
    let mut parts = vec![0, 0, 0];
    for i in [0, 1, 2] {
        if let Some(it) = captures.get(i + 1) {
            parts[i] = it
                .as_str()
                .parse::<usize>()
                .map_err(|e| Error::InternalError(e.to_string()))?;
        }
    }

    Ok(parts)
}

impl PartialEq<Self> for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Version {}

impl PartialOrd<Self> for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match compare_release(self, other) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Greater,
        }
    }
}

/// A naive comparison of `Version` release parts.
///
/// Expects three parts [N,N,N] in `this.release` and `other.release`.
fn compare_release(this: &Version, other: &Version) -> Ordering {
    for (a, b) in [
        (this.release[0], other.release[0]),
        (this.release[1], other.release[1]),
        (this.release[2], other.release[2]),
    ] {
        if a != b {
            return a.cmp(&b);
        }
    }

    Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_ord() {
        let (a, b) = (
            Version {
                release: vec![3, 10, 0],
            },
            Version {
                release: vec![3, 11, 0],
            },
        );
        assert!(a < b);
    }

    #[test]
    fn test_version_display() {
        let v = Version {
            release: vec![3, 11, 1],
        };
        assert_eq!(v.to_string(), "3.11.1");
    }
}
