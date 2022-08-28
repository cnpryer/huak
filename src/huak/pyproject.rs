use std::fmt;

struct Main {
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

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, Debug)]
enum DependencyKind {
    Main,
    Dev,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Dependency {
    name: String,
    version: String,
}

impl fmt::Display for Dependency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = \"{}\"", self.name, self.version)
    }
}

type DependencyList = Vec<Dependency>;

struct Dependencies {
    kind: DependencyKind,
    list: DependencyList,
}

impl fmt::Display for Dependencies {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = match self.kind {
            DependencyKind::Main => "",
            DependencyKind::Dev => "dev-",
        };

        writeln!(f, "[tool.huak.{}dependencies]", prefix)?;

        for dep in &self.list {
            writeln!(f, "{}", dep)?;
        }

        Ok(())
    }
}

struct BuildSystem {
    requires: Vec<String>,
    backend: String,
}

impl fmt::Display for BuildSystem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "[build-system]")?;
        writeln!(f, "requires = {:#?}", self.requires)?;
        writeln!(f, "build-backend = \"{}\"", self.backend)
    }
}

struct Toml {
    main: Main,
    dependencies: Dependencies,
    dev_dependencies: Dependencies,
    build_system: BuildSystem,
}

impl fmt::Display for Toml {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.main)?;
        writeln!(f)?;
        writeln!(f, "{}", self.dependencies)?;
        writeln!(f)?;
        writeln!(f, "{}", self.dev_dependencies)?;
        writeln!(f)?;
        writeln!(f, "{}", self.build_system)
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

    #[test]
    fn dependencies() {
        let kind = DependencyKind::Main;
        let list = vec![];
        let string = "\
[tool.huak.dependencies]
";

        let data = Dependencies {
            kind: kind.clone(),
            list: list.clone(),
        };

        assert_eq!(data.kind, kind);
        assert_eq!(data.list, list);
        assert_eq!(data.to_string(), string)
    }

    #[test]
    fn dev_dependencies() {
        let kind = DependencyKind::Dev;
        let list = vec![];
        let string = "\
[tool.huak.dev-dependencies]
";

        let data = Dependencies {
            kind: kind.clone(),
            list: list.clone(),
        };

        assert_eq!(data.kind, kind);
        assert_eq!(data.list, list);
        assert_eq!(data.to_string(), string);
    }

    #[test]
    fn build_system() {
        let requires = vec![];
        let backend = "".to_string();
        let string = "\
[build-system]
requires = []
build-backend = \"\"
";

        let data = BuildSystem {
            requires: requires.clone(),
            backend: backend.clone(),
        };

        assert_eq!(data.requires, requires);
        assert_eq!(data.backend, backend);
        assert_eq!(data.to_string(), string);
    }
}
