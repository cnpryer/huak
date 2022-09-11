use std::{fs, path::Path};

use super::{build_system::BuildSystem, tools::Tool};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Toml {
    pub(crate) tool: Tool,
    #[serde(rename = "build-system")]
    pub(crate) build_system: BuildSystem,
}

impl Toml {
    pub(crate) fn from(string: &str) -> Result<Toml, toml::de::Error> {
        toml::from_str(string)
    }

    pub(crate) fn open(path: &Path) -> Result<Toml, anyhow::Error> {
        let toml = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                return Err(anyhow::format_err!(
                    "failed to read toml file from {}",
                    path.display()
                ))
            }
        };

        let toml = match Toml::from(&toml) {
            Ok(t) => t,
            Err(_) => return Err(anyhow::format_err!("failed to build toml")),
        };

        Ok(toml)
    }

    pub(crate) fn to_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let string = r#"[tool.huak]
name = "Test"
version = "0.1.0"
description = ""
authors = []

[tool.huak.dependencies]

[tool.huak.dev-dependencies]

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;
        let toml = Toml::from(string).unwrap();

        assert_eq!(toml.to_string().unwrap(), string);
    }

    #[test]
    fn deserialize() {
        let string = r#"[tool.huak]
name = "Test"
version = "0.1.0"
description = ""
authors = []

[tool.huak.dependencies]

[tool.huak.dev-dependencies]

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;
        let toml = Toml::from(string).unwrap();

        assert_eq!(toml.tool.huak.name, "Test");
    }
}
