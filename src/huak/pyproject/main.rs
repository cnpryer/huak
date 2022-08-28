use std::fmt;

pub(crate) struct Main {
    name: String,
    version: String,
    description: String,
    authors: Vec<String>,
}

impl Default for Main {
    fn default() -> Main {
        Main {
            name: "".to_string(),
            version: "0.0.1".to_string(),
            description: "".to_string(),
            authors: vec![],
        }
    }
}

#[allow(dead_code)]
impl Main {
    pub fn new() -> Main {
        Main::default()
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

    pub fn add_author(&mut self, author: String) {
        self.authors.push(author)
    }
}

impl fmt::Display for Main {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "[tool.huak]")?;
        writeln!(f, "name = \"{}\"", self.name)?;
        writeln!(f, "version = \"{}\"", self.version)?;
        writeln!(f, "description = \"{}\"", self.description)?;
        writeln!(f, "authors = {:?}", self.authors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main() {
        let name = "Test".to_string();
        let version = "0.0.0".to_string();
        let description = "".to_string();
        let authors = vec![];
        let string = "\
[tool.huak]
name = \"Test\"
version = \"0.0.0\"
description = \"\"
authors = []
";

        let data = Main {
            name: name.clone(),
            version: version.clone(),
            description: description.clone(),
            authors: authors.clone(),
        };

        assert_eq!(data.name, name);
        assert_eq!(data.version, version);
        assert_eq!(data.description, description);
        assert_eq!(data.authors, authors);
        assert_eq!(data.to_string(), string);
    }
}
