use serde_derive::{Deserialize, Serialize};
use toml::{value::Map, Value};

use crate::Dependency;

#[derive(Serialize, Deserialize)]
pub struct Huak {
    name: String,
    version: String,
    description: String,
    authors: Vec<String>,
    dependencies: Map<String, Value>,
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: Map<String, Value>,
}

impl Default for Huak {
    fn default() -> Huak {
        Huak {
            name: "".to_string(),
            version: "0.0.1".to_string(),
            description: "".to_string(),
            authors: vec![],
            dependencies: Map::new(),
            dev_dependencies: Map::new(),
        }
    }
}

impl Huak {
    pub fn new() -> Huak {
        Huak::default()
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    pub fn version(&self) -> &String {
        &self.version
    }

    pub fn set_version(&mut self, version: String) {
        self.version = version
    }

    pub fn description(&self) -> &String {
        &self.description
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description
    }

    pub fn authors(&self) -> &Vec<String> {
        &self.authors
    }

    pub fn set_authors(&mut self, authors: Vec<String>) {
        self.authors = authors;
    }

    pub fn add_author(&mut self, author: String) {
        self.authors.push(author);
    }

    pub fn dependencies(&self) -> &Map<String, Value> {
        &self.dependencies
    }

    pub fn add_dependency(&mut self, dependency: Dependency) {
        self.dependencies
            .insert(dependency.name, Value::String(dependency.version));
    }

    pub fn set_dependencies(&mut self, dependencies: Vec<Dependency>) {
        for dependency in dependencies {
            self.add_dependency(dependency);
        }
    }

    // Remove dependency from either of the main or dev dependency lists.
    pub fn remove_dependency(&mut self, name: &str) {
        self.remove_main_dependency(name);
        self.remove_dev_dependency(name);
    }

    pub fn remove_main_dependency(&mut self, name: &str) {
        self.dependencies.remove(name);
    }

    pub fn dev_dependencies(&self) -> &Map<String, Value> {
        &self.dev_dependencies
    }

    pub fn add_dev_dependency(&mut self, dependency: Dependency) {
        self.dev_dependencies
            .insert(dependency.name, Value::String(dependency.version));
    }

    pub fn set_dev_dependencies(&mut self, dependencies: Vec<Dependency>) {
        for dependency in dependencies {
            self.add_dev_dependency(dependency);
        }
    }

    pub fn remove_dev_dependency(&mut self, name: &str) {
        self.dev_dependencies.remove(name);
    }
}
