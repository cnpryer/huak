use std::fmt;

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum DependencyKind {
    Main,
    Dev,
}

#[allow(dead_code)]
fn match_dependency_kind(kind: &str) -> DependencyKind {
    if let "dev" = kind {
        DependencyKind::Dev
    } else {
        DependencyKind::Main
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub(crate) struct Dependency {
    name: String,
    version: String,
}

#[allow(dead_code)]
impl Dependency {
    pub fn new(name: String, version: String) -> Dependency {
        Dependency { name, version }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn version(&self) -> &String {
        &self.version
    }
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

#[allow(dead_code)]
impl Dependencies {
    pub fn new(kind: &str) -> Dependencies {
        Dependencies {
            kind: match_dependency_kind(kind),
            list: vec![],
        }
    }

    pub fn kind(&self) -> &DependencyKind {
        &self.kind
    }

    pub fn set_kind(&mut self, kind: &str) {
        self.kind = match_dependency_kind(kind)
    }

    pub fn list(&self) -> &DependencyList {
        &self.list
    }

    pub fn add_dependency(&mut self, dependency: Dependency) {
        self.list.push(dependency)
    }
}

impl Default for Dependencies {
    fn default() -> Dependencies {
        Dependencies {
            kind: DependencyKind::Main,
            list: vec![],
        }
    }
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
