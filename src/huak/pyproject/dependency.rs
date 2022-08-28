use std::fmt;

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

pub(crate) struct Dependencies {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
