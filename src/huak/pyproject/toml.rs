use std::fmt;

use super::{
    build_system::BuildSystem,
    dependency::{Dependencies, Dependency},
    main::Main,
};

pub struct Toml {
    main: Main,
    dependencies: Dependencies,
    dev_dependencies: Dependencies,
    build_system: BuildSystem,
}

#[allow(dead_code)]
impl Toml {
    pub fn new(main: Main) -> Toml {
        Toml {
            main,
            dependencies: Dependencies::default(),
            dev_dependencies: Dependencies::new("dev"),
            build_system: BuildSystem::default(),
        }
    }

    pub fn main(&self) -> &Main {
        &self.main
    }

    pub fn set_main(&mut self, main: Main) {
        self.main = main
    }

    pub fn dependencies(&self) -> &Dependencies {
        &self.dependencies
    }

    pub fn dev_dependencies(&self) -> &Dependencies {
        &self.dev_dependencies
    }

    pub fn add_dependency(&mut self, dependency: Dependency, kind: &str) {
        if let "main" = kind {
            self.dependencies.add_dependency(dependency)
        } else {
            self.dev_dependencies.add_dependency(dependency)
        }
    }
}

impl fmt::Display for Toml {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.main)?;
        writeln!(f, "{}", self.dependencies)?;
        writeln!(f, "{}", self.dev_dependencies)?;
        write!(f, "{}", self.build_system)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml() {
        let mut main = Main::default();
        main.set_name("Test".to_string());
        main.set_version("0.1.0".to_string());

        let toml = Toml::new(main);
        let string = "\
[tool.huak]
name = \"Test\"
version = \"0.1.0\"
description = \"\"
authors = []

[tool.huak.dependencies]

[tool.huak.dev-dependencies]

[build-system]
requires = [\"huak-core>=1.0.0\"]
build-backend = \"huak.core.build.api\"
";

        assert_eq!(toml.to_string(), string);
    }
}
