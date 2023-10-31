use crate::{
    error::Error,
    releases::{Release, RELEASES},
    Version,
};
use std::{
    env::consts::{ARCH, OS},
    fmt::Display,
    str::FromStr,
};

/// Resolve a Python Release based on a resolution `Strategy`.
#[must_use]
pub fn resolve_release(strategy: &Strategy) -> Option<Release<'static>> {
    match strategy {
        Strategy::Latest => resolve_release_with_options(&Options::default()),
        Strategy::Selection(options) => resolve_release_with_options(options),
    }
}

fn resolve_release_with_options(options: &Options) -> Option<Release<'static>> {
    let mut candidates = RELEASES
        .iter()
        .filter(|it| {
            it.kind == options.kind
                && it.os == options.os
                && it.architecture == options.architecture
                && it.build_configuration == options.build_configuration
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        None
    } else {
        // Sort releases by version in descending order (latest releases at the beginning of the vector)
        candidates.sort_by(|a, b| b.version.cmp(&a.version));

        if let Some(req) = options.version.as_ref() {
            candidates
                .into_iter()
                .find(|it| req.matches_version(&it.version))
                .copied()
        } else {
            candidates.first().map(|it| **it)
        }
    }
}

/// The strategy used for resolving a Python releases.
#[derive(Default)]
pub enum Strategy<'a> {
    #[default]
    /// Resolve with the latest possible Python release version for the current environment.
    Latest,
    /// `Selection` - Use some selection criteria to determine the Python release. Unused
    /// options criteria will resolve to *best possible defaults*.
    Selection(Options<'a>),
}

/// Options criteria used for resolving Python releases.
#[derive(Debug)]
pub struct Options<'a> {
    pub kind: &'a str,
    pub version: Option<RequestedVersion>, // TODO(cnpryer): Refactor to default as *latest available*
    pub os: &'a str,
    pub architecture: &'a str,
    pub build_configuration: &'a str,
}

// TODO(cnpryer): Refactor
impl Default for Options<'static> {
    fn default() -> Self {
        Self {
            kind: "cpython",
            version: Option::default(),
            os: match OS {
                "macos" => "apple",
                "windows" => "windows",
                _ => "linux",
            },
            architecture: match ARCH {
                "x86_64" => "x86_64",
                "aarch64" => "aarch64",
                "x86" => "i686", // TODO(cnpryer): Need to look at other windows releases.
                _ => unimplemented!(),
            },
            build_configuration: match OS {
                "windows" => "pgo",
                _ => "pgo+lto",
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestedVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: Option<u8>,
}

impl RequestedVersion {
    /// Evaluates if some Python release's version is what was requested.
    #[must_use]
    pub fn matches_version(&self, version: &Version) -> bool {
        self.major == version.major
            && self.minor == version.minor
            && self.patch.map_or(true, |it| Some(it) == version.patch)
    }
}

impl FromStr for RequestedVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.').map(str::parse);

        let Some(Ok(major)) = parts.next() else {
            return Err(Error::ParseRequestedVersionError(s.to_string()));
        };

        let Some(Ok(minor)) = parts.next() else {
            return Err(Error::ParseRequestedVersionError(s.to_string()));
        };

        let patch = match parts.next() {
            Some(Ok(it)) => Some(it),
            Some(Err(_e)) => return Err(Error::ParseRequestedVersionError(s.to_string())),
            _ => None,
        };

        Ok(RequestedVersion {
            major,
            minor,
            patch,
        })
    }
}

impl Display for RequestedVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)?;

        if let Some(patch) = self.patch {
            write!(f, ".{patch}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_latest() {
        let latest_default = resolve_release_with_options(&Options::default()).unwrap();
        let resolved_release = resolve_release(&Strategy::Latest).unwrap();

        assert_eq!(resolved_release, latest_default);
    }

    #[test]
    fn test_selection() {
        let resolved_release = resolve_release(&Strategy::Selection(Options {
            kind: "cpython",
            version: Some(RequestedVersion::from_str("3.8").unwrap()),
            os: "apple",
            architecture: "aarch64",
            build_configuration: "pgo+lto",
        }))
        .unwrap();

        assert_eq!(resolved_release.kind, "cpython");
        assert_eq!(resolved_release.version.major, 3);
        assert_eq!(resolved_release.version.minor, 8);
        assert_eq!(resolved_release.os, "apple");
        assert_eq!(resolved_release.architecture, "aarch64");
        assert_eq!(resolved_release.build_configuration, "pgo+lto");
    }
}
