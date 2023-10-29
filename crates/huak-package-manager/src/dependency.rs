use std::{ffi::OsStr, fmt::Display, str::FromStr};

use pep440_rs::VersionSpecifiers;
use pep508_rs::{Requirement, VersionOrUrl};

use crate::Error;

/// The `Dependency` is an abstraction for `Package` data used as a cheap alternative
/// for operations on lots of `Package` data.
///
/// `Dependency`s can contain different information about a `Package` necessary to
/// use them as `Package` `Dependency`s, such as having multiple `VersionSpecifiers`.
///
/// ```
/// use huak_package_manager::Dependency;
///
/// let dependency = Dependency::from_str("my-dependency >= 0.1.0, < 0.2.0").unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct Dependency(Requirement);

impl Dependency {
    /// Get a reference to the wrapped `Requirement`.
    #[must_use]
    pub fn requirement(&self) -> &Requirement {
        &self.0
    }

    /// Get a mutable reference to the wrapped `Requirement`.
    pub fn requirement_mut(&mut self) -> &mut Requirement {
        &mut self.0
    }

    /// Get the `Dependency` name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.requirement().name
    }

    /// Get a reference to the `Dependency`'s `VersionSpecifiers`.
    #[allow(dead_code)]
    fn version_specifiers(&self) -> Option<&VersionSpecifiers> {
        match self.0.version_or_url.as_ref() {
            Some(VersionOrUrl::VersionSpecifier(it)) => Some(it),
            _ => None,
        }
    }
}

impl From<Requirement> for Dependency {
    fn from(value: Requirement) -> Self {
        Dependency(value)
    }
}

/// Initialize a `Dependency` from a `&str`.
///
/// ```
/// use huak_package_manager::Dependency;
///
/// let dependency = Dependency::from_str("my-dependency >= 0.1.0, < 0.2.0").unwrap();
/// ```
impl FromStr for Dependency {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let requirement = Requirement::from_str(s)?;
        let dependency = Dependency(requirement);

        Ok(dependency)
    }
}

impl Display for Dependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.requirement())
    }
}

impl From<&Requirement> for Dependency {
    fn from(value: &Requirement) -> Self {
        Dependency(value.clone())
    }
}

impl AsRef<OsStr> for Dependency {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self)
    }
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for Dependency {}

/// Construct an `Iterator` over an `IntoIterator` of `&str`s.
///
/// ```
/// let dependencies = vec!["my-dep", "my-dep==0.0.1"];
/// let iter = dependency_iter(dependencies);
/// ```
pub fn dependency_iter<I>(iter: I) -> impl Iterator<Item = Dependency>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    iter.into_iter()
        .filter_map(|item| Dependency::from_str(item.as_ref()).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_from_str() {
        let dep = Dependency::from_str("package-name==0.0.0").unwrap();

        assert_eq!(dep.to_string(), "package-name ==0.0.0");
        assert_eq!(dep.name(), "package-name");
        assert_eq!(
            *dep.version_specifiers().unwrap(),
            pep440_rs::VersionSpecifiers::from_str("==0.0.0").unwrap()
        );
    }
}
