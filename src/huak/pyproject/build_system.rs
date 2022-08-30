use serde_derive::{Deserialize, Serialize};
use std::fmt;

const HUAK_REQUIRES: &str = "huak-core>=1.0.0";
const HUAK_BUILD_BACKEND: &str = "huak.core.build.api";

#[derive(Serialize, Deserialize)]
pub struct BuildSystem {
    requires: Vec<String>,
    #[serde(rename = "build-backend")]
    backend: String,
}

impl Default for BuildSystem {
    fn default() -> BuildSystem {
        BuildSystem {
            requires: vec![HUAK_REQUIRES.to_string()],
            backend: HUAK_BUILD_BACKEND.to_string(),
        }
    }
}

impl fmt::Display for BuildSystem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "[build-system]")?;
        writeln!(f, "requires = {:?}", self.requires)?;
        writeln!(f, "build-backend = \"{}\"", self.backend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_system() {
        let requires = vec![];
        let backend = "".to_string();
        let string = "\
[build-system]
requires = []
build-backend = \"\"
";

        let data = BuildSystem {
            requires: requires.clone(),
            backend: backend.clone(),
        };

        assert_eq!(data.requires, requires);
        assert_eq!(data.backend, backend);
        assert_eq!(data.to_string(), string);
    }
}
