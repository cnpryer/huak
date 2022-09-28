use std::{fs, path::Path};

use super::{build_system::BuildSystem, project::Project};
use serde_derive::{Deserialize, Serialize};

/// Toml configuration deser and ser structure.
/// ```toml
/// [tool.huak]
/// # ...
/// [tool.huak.dependencies]
/// # ...
/// [tool.huak.dev-dependencies]
/// # ...
/// [tool.build-system]
/// # ...
/// ```
#[derive(Serialize, Deserialize, Default)]
pub struct Toml {
    pub(crate) project: Project,
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

impl Toml {
    pub fn add_dependency(&mut self, dependency: &str) {
        self.project.dependencies.push(dependency.to_string());
    }

    pub fn add_optional_dependency(&mut self, dependency: &str) {
        match &mut self.project.optional_dependencies {
            Some(deps) => {
                deps.push(dependency.to_string());
            }
            None => {
                self.project.optional_dependencies =
                    Some(vec![dependency.to_string()]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let string = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = ["click==8.1.3", "black==22.8.0"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;
        let toml = Toml::from(string).unwrap();

        let res = toml.to_string().unwrap();
        dbg!(&res);

        assert_eq!(res, string);
    }

    #[test]
    fn deserialize() {
        let string = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = ["click==8.1.3", "black==22.8.0"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;
        let toml = Toml::from(string).unwrap();

        assert_eq!(toml.project.name, "Test");
        assert_eq!(
            toml.project.authors[0].clone().name.unwrap(),
            "Chris Pryer"
        );
    }

    #[test]
    fn deserialize_array_of_authors() {
        let string = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = ["click==8.1.3", "black==22.8.0"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[[project.authors]]
name = "Troy Kohler"
email = "test@email.com"

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;

        let toml = Toml::from(string).unwrap();

        assert!(toml.project.authors.iter().nth(1).is_some());
    }
}
