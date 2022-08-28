use std::fmt;

use super::{build_system::BuildSystem, dependency::Dependencies, main::Main};

struct Toml {
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
