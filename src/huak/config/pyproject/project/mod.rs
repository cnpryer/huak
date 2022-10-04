use pyproject_toml::Project;
/*
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
*/
/*
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
    pub(crate) optional_dependencies: Option<HashMap<String, Vec<String>>>,
    pub(crate) authors: Vec<Author>,
    pub(crate) scripts: Option<HashMap<String, String>>,
}
*/
/// Builder struct to create create default Project data.
pub struct ProjectBuilder {}

impl ProjectBuilder {
    pub fn default() -> Project {
        Project {
            name: "".to_string(),
            version: Some("0.0.1".to_string()),
            description: Some("".to_string()),
            readme: None,
            requires_python: None,
            license: None,
            authors: Some(vec![]),
            maintainers: None,
            keywords: None,
            classifiers: None,
            urls: None,
            entry_points: None,
            scripts: None,
            gui_scripts: None,
            dependencies: Some(vec![]),
            optional_dependencies: None,
            dynamic: None,
        }
    }
}
