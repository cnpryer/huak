use std::{fs, path::Path};

use serde_derive::{Deserialize, Serialize};
use pyproject_toml::{Project, BuildSystem};

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
/*
#[derive(Serialize, Deserialize, Default)]
pub struct Toml {
    pub(crate) project: Project,
    #[serde(rename = "build-system")]
    pub(crate) build_system: BuildSystem,
}
*/
#[derive(Serialize, Deserialize, Default)]
pub struct Toml {
    pub(crate) project: ProjectWrapper,
    pub(crate) build_system: BuildSystemWrapper,
}

impl Toml {
    pub(crate) fn from(string: &str) -> Result<Toml, toml::de::Error> {
        toml::from(string)
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
        match self.project.dependencies {
            Some(dependencies) => {
                dependencies.push(dependency.to_string());
            },
            None => {
                self.project.dependencies = Some(vec![dependency.to_string()]);
            }
        }
    }

    pub fn add_optional_dependency(&mut self, name: &str, dependencies: &Vec<String>) {
        unimplemented!()
    }

    pub fn remove_dependency(&mut self, dependency: &str) {
        // TODO: Do better than .starts_with
        if let Some(deps) = self.project.dependencies {
            dependencies.retain(|s| !s.starts_with(dependency));
        }
    }

    pub fn remove_optional_dependency(&mut self, dependency: &str) {
        if let Some(deps) = &mut self.project.optional_dependencies {
            deps.retain(|(k, v)| !k.starts_with(dependency));
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
