use super::{build_system::BuildSystem, tools::Tool};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Toml {
    pub tool: Tool,
    #[serde(rename = "build-system")]
    pub build_system: BuildSystem,
}

impl Toml {
    pub fn new() -> Toml {
        Toml::default()
    }

    pub fn from(string: &str) -> Result<Toml, toml::de::Error> {
        toml::from_str(string)
    }

    pub fn tool(&self) -> &Tool {
        &self.tool
    }

    pub fn set_tool(&mut self, tool: Tool) {
        self.tool = tool;
    }

    pub fn to_string(&self) -> Result<String, toml::ser::Error> {
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

        assert_eq!(toml.tool.huak.name(), "Test");
    }
}
