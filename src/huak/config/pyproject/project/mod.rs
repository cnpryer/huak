use serde_derive::{Deserialize, Serialize};

/// Struct containing Author information.
/// ```toml
/// [[project.authors]]
/// name = "Chris Pryer"
/// email = "cnpryer@gmail.com"
/// ```
#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct Author {
    pub(crate) name: Option<String>,
    pub(crate) email: Option<String>,
}

/// Project table data.
/// ```toml
/// [project]
/// name = "Project"
/// version = "0.0.1"
/// description = ""
/// # ...
/// ```
#[derive(Serialize, Deserialize)]
pub(crate) struct Project {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) description: String,
    pub(crate) dependencies: Vec<String>,
    #[serde(rename = "optional-dependencies")]
    pub(crate) optional_dependencies: Option<Vec<String>>,
    pub(crate) authors: Vec<Author>,
}

impl Default for Project {
    fn default() -> Project {
        Project {
            name: "".to_string(),
            version: "0.0.1".to_string(),
            description: "".to_string(),
            authors: vec![],
            dependencies: vec![],
            optional_dependencies: Some(vec![]),
        }
    }
}
