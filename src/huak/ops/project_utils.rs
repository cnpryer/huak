use std::collections::HashMap;

use crate::{
    config::pyproject::toml::Toml,
    project::{Project, ProjectType},
};

/// Create project toml.
// TODO: Config implementations?
pub fn create_toml(project: &Project) -> Result<Toml, anyhow::Error> {
    let mut toml = Toml::default();
    let name = crate::utils::path::parse_filename(&project.root)?.to_string();

    if matches!(project.project_type, ProjectType::Application) {
        let entrypoint = format!("{name}:run");
        toml.project.scripts = Some(HashMap::from([(name.clone(), entrypoint)]))
    }

    toml.project.name = name;

    Ok(toml)
}
