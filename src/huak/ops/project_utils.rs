use std::collections::HashMap;

use crate::{
    config::pyproject::toml::Toml,
    errors::HuakResult,
    project::{Project, ProjectType},
};

/// Create project toml.
// TODO: Config implementations?
pub fn create_toml(project: &Project) -> HuakResult<Toml> {
    let mut toml = Toml::default();
    let name = crate::utils::path::parse_filename(&project.root)?.to_string();

    if matches!(project.project_type, ProjectType::Application) {
        let entrypoint = format!("{name}:run");
        toml.project.scripts = Some(HashMap::from([(name.clone(), entrypoint)]))
    }

    toml.project.name = name;

    Ok(toml)
}
