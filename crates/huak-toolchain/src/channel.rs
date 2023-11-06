use huak_python_manager::Version;
use std::{fmt::Display, str::FromStr};

use crate::Error;

#[derive(Default, Clone, Debug)]
pub enum Channel {
    #[default]
    Default,
    Version(Version),
    Descriptor(DescriptorParts),
}

/// Parse `Channel` from strings. This is useful for parsing channel inputs for applications implementing CLI.
impl FromStr for Channel {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "default" {
            return Ok(Self::Default);
        }

        let Ok(version) = Version::from_str(s) else {
            return Err(Error::ParseChannelError(s.to_string()));
        };

        Ok(Channel::Version(version))
    }
}

impl Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::Default => write!(f, "default"),
            Channel::Version(version) => write!(f, "{version}"),
            Channel::Descriptor(desc) => write!(f, "{desc}"),
        }
    }
}

// Right now this is just a dynamic struct of `Release` data.
#[derive(Clone, Debug)]
pub struct DescriptorParts {
    pub kind: Option<String>,
    pub version: Option<Version>,
    pub os: Option<String>,
    pub architecture: Option<String>,
    pub build_configuration: Option<String>,
}

impl Display for DescriptorParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Only allocate enough for `DescriptorParts` data.
        let mut parts = Vec::with_capacity(5);

        if let Some(kind) = &self.kind {
            parts.push(kind.to_string());
        }

        if let Some(version) = &self.version {
            parts.push(format!("{version}"));
        }

        if let Some(os) = &self.os {
            parts.push(os.to_string());
        }

        if let Some(architecture) = &self.architecture {
            parts.push(architecture.to_string());
        }

        if let Some(build_config) = &self.build_configuration {
            parts.push(build_config.to_string());
        }

        write!(f, "{}", parts.join("-"))
    }
}
