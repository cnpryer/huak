use std::fmt;

use super::{build_system::BuildSystem, dependency::Dependencies, main::Main};

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
