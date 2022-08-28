use std::fmt;

pub(crate) struct Main {
    name: String,
    version: String,
    description: String,
    authors: Vec<String>,
}

impl fmt::Display for Main {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "[tool.huak]")?;
        writeln!(f, "name = \"{}\"", self.name)?;
        writeln!(f, "version = \"{}\"", self.version)?;
        writeln!(f, "description = \"{}\"", self.description)?;
        writeln!(f, "authors = {:#?}", self.authors)
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
