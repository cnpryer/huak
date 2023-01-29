use pyproject_toml::Project;

#[derive(Clone)]
/// Builder struct to create load default Project data into pyproject-toml-rs.
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
