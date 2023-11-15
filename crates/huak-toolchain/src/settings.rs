//! This module implements read and write functionality for Huak's persisted application data.
use crate::Error;
use std::path::{Path, PathBuf};
use toml_edit::Document;

#[derive(Default)]
pub struct SettingsDb {
    doc: Document, // TODO(cnpryer): Decouple from toml_edit here
}

impl SettingsDb {
    #[must_use]
    pub fn new() -> Self {
        Self {
            doc: Document::new(),
        }
    }

    #[must_use]
    pub fn doc(&self) -> &Document {
        &self.doc
    }

    pub fn doc_mut(&mut self) -> &mut Document {
        &mut self.doc
    }

    pub fn try_from<T: AsRef<Path>>(path: T) -> Result<Self, Error> {
        let mut db = Self::new();
        db.doc = read_settings_file(path)?;
        Ok(db)
    }

    /// Insert a scope entry.
    ///
    /// ```rust
    /// use huak_toolchain::SettingsDb;
    /// use std::path::PathBuf;
    ///
    /// let mut db = SettingsDb::new();
    /// let cwd = PathBuf::new();
    /// let toolchain = PathBuf::new();
    ///
    /// db.insert_scope(cwd, toolchain);
    /// ```
    pub fn insert_scope<T: AsRef<Path>>(&mut self, key: T, value: T) -> Result<(), Error> {
        let key_string = dunce::canonicalize(key)?.to_string_lossy().to_string();
        let value_string = dunce::canonicalize(value)?.to_string_lossy().to_string();

        self.doc_mut()["scopes"][key_string] = toml_edit::value(value_string);

        Ok(())
    }

    pub fn remove_scope<T: AsRef<Path>>(&mut self, key: T) -> Result<(), Error> {
        let key_string = dunce::canonicalize(key.as_ref())?
            .to_string_lossy()
            .to_string();

        self.doc_mut()
            .get_mut("scopes")
            .and_then(|it| it.as_inline_table_mut()) // TODO(cnpryer): Don't inline
            .and_then(|it| it.remove(&key_string));

        Ok(())
    }

    // TODO(cnpryer): Potentially use `ScopeEntry`.
    pub fn get_scope_entry<T: AsRef<Path>>(&self, key: T) -> Result<Option<(T, String)>, Error> {
        let key_string = dunce::canonicalize(key.as_ref())?
            .to_string_lossy()
            .to_string();

        // TODO(cnpryer): Smarter escape
        let entry = self.doc().get("scopes").and_then(|it| {
            it.get(key_string)
                .map(|v| (key, escape_str(&v.to_string())))
        });

        Ok(entry)
    }

    pub fn save<T: AsRef<Path>>(&self, to: T) -> Result<(), Error> {
        write_settings_file(self.doc(), to)
    }

    pub fn remove_toolchain<T: AsRef<Path>>(&mut self, path: T) -> Result<(), Error> {
        if let Some(scopes) = self.doc().get("scopes") {
            if let Some(values) = scopes.as_inline_table().map(|it| it.get_values()) {
                let key_path = dunce::canonicalize(path.as_ref())?;
                // TODO(cnpryer): Perf
                let keys_to_remove = values
                    .iter()
                    .filter(|(_, v)| PathBuf::from(escape_str(&v.to_string())) == key_path)
                    .flat_map(|(keys, _)| {
                        keys.iter()
                            .map(|k| PathBuf::from(escape_str(&k.to_string())))
                    })
                    .collect::<Vec<PathBuf>>();

                for key in keys_to_remove {
                    self.remove_scope(key)?;
                }
            }
        }

        Ok(())
    }
}

/// A helper for reading the contents of a settings.toml file.
pub(crate) fn read_settings_file<T: AsRef<Path>>(path: T) -> Result<Document, Error> {
    let doc = std::str::from_utf8(std::fs::read(path)?.as_slice())?.parse::<Document>()?;

    Ok(doc)
}

pub(crate) fn write_settings_file<T: AsRef<Path>>(doc: &Document, path: T) -> Result<(), Error> {
    Ok(std::fs::write(path, doc.to_string())?)
}

pub fn escape_str(s: &str) -> String {
    let pattern = ['\\', '\'', '"'];
    s.trim()
        .trim_start_matches(pattern)
        .trim_end_matches(&pattern)
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::fs::create_dir_all;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_scopes() {
        let mut db = SettingsDb::new();
        let dir = TempDir::new().unwrap();
        let dir = dir.path();
        let project1 = dir.join("project1");
        let project2 = dir.join("project2");
        let toolchain = dir.join("toolchain");

        for p in [&project1, &project2, &toolchain] {
            create_dir_all(p).unwrap();
        }

        db.insert_scope(dir, &toolchain).unwrap();

        let (_, value) = db.get_scope_entry(&dir).unwrap().unwrap();

        assert_eq!(
            PathBuf::from(value),
            dunce::canonicalize(&toolchain).unwrap()
        );

        db.remove_scope(dir).unwrap();

        let table = db.doc().get("scopes").unwrap();

        assert!(table
            .as_inline_table()
            .map_or(false, toml_edit::InlineTable::is_empty));
    }

    #[test]
    fn test_remove_toolchain() {
        let mut db = SettingsDb::new();
        let dir = TempDir::new().unwrap();
        let dir = dir.path();
        let project1 = dir.join("project1");
        let project2 = dir.join("project2");
        let toolchain = dir.join("toolchain");

        for p in [&project1, &project2, &toolchain] {
            create_dir_all(p).unwrap();
        }

        db.insert_scope(&project1, &toolchain).unwrap();
        db.insert_scope(&project2, &toolchain).unwrap();

        let (_, value) = db.get_scope_entry(project1).unwrap().unwrap();

        assert_eq!(
            PathBuf::from(value),
            dunce::canonicalize(&toolchain).unwrap()
        );

        db.remove_toolchain(&toolchain).unwrap();

        let table = db.doc().get("scopes").unwrap();

        assert!(table
            .as_inline_table()
            .map_or(false, toml_edit::InlineTable::is_empty));
    }
}
