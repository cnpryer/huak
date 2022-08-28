use std::fmt;

pub(crate) struct BuildSystem {
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

#[cfg(test)]
mod tests {
    use super::*;

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
