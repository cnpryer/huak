use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::{cmp::Ordering, fmt::Display, str::FromStr};

use crate::error::Error;

lazy_static! {
    static ref VERSION_REGEX: Regex =
        Regex::new(r"^(\d+)(?:\.(\d+))?(?:\.(\d+))?$").expect("version regex");
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: Option<u8>,
}

impl Version {
    #[must_use]
    pub const fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch: Some(patch),
        }
    }
}

/// Initialize a `Version` from a `&str`.
///
/// ```rust
/// use std::str::FromStr;
/// use huak_python_manager::Version;
///
/// let a = Version::from_str("0.0.1").unwrap();
/// let b = Version::from_str("0.0.2").unwrap();
///
/// assert!(a < b);
/// ```
impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_version_from_captures(&captures_version_str(s)?)
    }
}

/// Use regex to capture potential `Version` numbers from a `&str`.
fn captures_version_str(s: &str) -> Result<Captures, Error> {
    let Some(captures) = VERSION_REGEX.captures(s) else {
        return Err(Error::InvalidVersion(s.to_string()));
    };

    Ok(captures)
}

/// Parse `Version` from a regex capture of version parts.
fn parse_version_from_captures(captures: &Captures) -> Result<Version, Error> {
    let mut parts = [None, None, None];

    for i in [0, 1, 2] {
        if let Some(it) = captures.get(i + 1) {
            parts[i] = Some(
                it.as_str()
                    .parse::<u8>()
                    .map_err(|e| Error::InvalidVersion(e.to_string()))?,
            );
        }
    }

    let Some(major) = parts[0] else {
        return Err(Error::InvalidVersion("missing major".to_string()));
    };

    let Some(minor) = parts[1] else {
        return Err(Error::InvalidVersion("missing minor".to_string()));
    };

    Ok(Version {
        major,
        minor,
        patch: parts[2],
    })
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)?;

        if let Some(patch) = self.patch {
            write!(f, ".{patch}")?;
        }

        Ok(())
    }
}

impl PartialOrd<Self> for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match compare_version(*self, *other) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Greater,
        }
    }
}

fn compare_version(this: Version, other: Version) -> Ordering {
    for (a, b) in [
        (this.major, other.major),
        (this.minor, other.minor),
        (
            this.patch.unwrap_or(u8::MAX),
            other.patch.unwrap_or(u8::MAX),
        ),
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
        let (a, b) = (Version::new(3, 10, 0), Version::new(3, 10, 1));

        assert!(a < b);
    }

    #[test]
    fn test_version_display() {
        let v = Version::new(3, 11, 1);

        assert_eq!(v.to_string(), "3.11.1");
    }
}
