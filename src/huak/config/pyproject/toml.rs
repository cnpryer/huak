use std::collections::HashMap;
use std::{fs, path::Path};

use crate::errors::{HuakError, HuakResult};
use pyproject_toml::{BuildSystem, Project};

use serde_derive::{Deserialize, Serialize};

use super::build_system::BuildSystemBuilder;
use super::project::ProjectBuilder;

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
#[derive(Serialize, Deserialize, Debug)]
pub struct Toml {
    #[serde(rename = "build-system")]
    pub build_system: BuildSystem,
    pub project: Project,
}

impl Default for Toml {
    fn default() -> Toml {
        Toml {
            build_system: BuildSystemBuilder::default(),
            project: ProjectBuilder::default(),
        }
    }
}

impl Toml {
    pub(crate) fn from(string: &str) -> HuakResult<Toml> {
        Ok(toml_edit::de::from_str(string)?)
    }

    pub(crate) fn open(path: &Path) -> HuakResult<Toml> {
        let toml = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                return Err(HuakError::InternalError(format!(
                    "failed to read toml file from {}",
                    path.display()
                )))
            }
        };

        dbg!(&toml);
        println!("{}", toml);

        let toml = match Toml::from(&toml) {
            Ok(t) => t,
            Err(_) => {
                return Err(HuakError::InternalError(
                    "failed to build toml".into(),
                ))
            }
        };

        Ok(toml)
    }

    pub(crate) fn to_string(&self) -> HuakResult<String> {
        Ok(toml_edit::ser::to_string_pretty(&self)?)
    }
}

impl Toml {
    pub fn add_dependency(&mut self, dependency: &str) {
        match &mut self.project.dependencies {
            Some(dependencies) => {
                dependencies.push(dependency.to_string());
            }
            None => {
                self.project.dependencies = Some(vec![dependency.to_string()]);
            }
        }
    }

    pub fn add_optional_dependency(&mut self, group: &str, dependency: &str) {
        match &mut self.project.optional_dependencies {
            Some(deps) => deps
                .entry(group.to_string())
                .or_insert_with(Vec::new)
                .push(dependency.to_string()),
            None => {
                self.project.optional_dependencies = Some(HashMap::from([(
                    group.to_string(),
                    vec![dependency.to_string()],
                )]));
            }
        }
    }

    pub fn remove_dependency(&mut self, dependency: &str) {
        // TODO: Do better than .starts_with
        if let Some(deps) = &mut self.project.dependencies {
            deps.retain(|s| !s.starts_with(dependency));
        }

        if let Some(deps) = &mut self.project.optional_dependencies {
            for (_, group_deps) in deps.iter_mut() {
                group_deps.retain(|s| !s.starts_with(dependency));
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

[project.optional-dependencies]
test = ["pytest>=6", "mock"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;

        // toml_edit does not preserve the ordering of the tables
        let expected_output = r#"[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"

[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = [
    "click==8.1.3",
    "black==22.8.0",
]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"
"#;

        let toml = Toml::from(string).unwrap();

        println!("{}", toml.project.name.clone());
        println!("{}", toml.project.version.as_ref().unwrap());

        let res = toml.to_string().unwrap();
        dbg!(&res);

        assert_eq!(expected_output, &res);
    }

    #[test]
    fn deserialize() {
        let string = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = ["click==8.1.3", "black==22.8.0"]

[project.optional-dependencies]
test = ["pytest>=6", "mock"]

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
            toml.project.authors.unwrap()[0]
                .name
                .as_ref()
                .unwrap()
                .clone(),
            "Chris Pryer"
        );

        assert_eq!(toml.build_system.requires, &["huak-core>=1.0.0"]);
        assert_eq!(
            toml.build_system.build_backend,
            Some(String::from("huak.core.build.api"))
        );

        assert_eq!(toml.project.version, Some(String::from("0.1.0")));
        assert_eq!(toml.project.description, Some(String::from("")));
        assert_eq!(
            toml.project.dependencies,
            Some(vec![
                String::from("click==8.1.3"),
                String::from("black==22.8.0")
            ])
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

        match Toml::from(string) {
            Ok(toml) => {
                println!("{:?}", toml)
            }
            Err(err) => {
                eprintln!("{}", err)
            }
        }

        let toml = Toml::from(string).unwrap();

        assert_eq!(
            "Troy Kohler",
            toml.project
                .authors
                .as_ref()
                .unwrap()
                .get(1)
                .unwrap()
                .name
                .as_ref()
                .unwrap()
        );
        assert_eq!(
            "test@email.com",
            toml.project
                .authors
                .as_ref()
                .unwrap()
                .get(1)
                .unwrap()
                .email
                .as_ref()
                .unwrap()
        );
    }
}
