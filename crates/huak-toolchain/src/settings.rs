//! This module implements read and write functionality for Huak's persisted application data.
use crate::Error;
use std::path::Path;
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
    /// let channel = "3.12";
    ///
    /// db.insert_scope(cwd, channel);
    /// ```
    pub fn insert_scope<T: AsRef<Path>>(&mut self, key: T, value: T) {
        let key = format!("{}", key.as_ref().display());
        let value = format!("{}", value.as_ref().display());

        self.doc_mut()["scopes"][key] = toml_edit::value(value);
    }

    pub fn remove_scope<T: AsRef<Path>>(&mut self, key: T) {
        let key = format!("{}", key.as_ref().display());

        self.doc_mut()
            .get_mut("scopes")
            .and_then(|it| it.as_inline_table_mut()) // TODO(cnpryer): Don't inline
            .and_then(|it| it.remove(&key));
    }

    // TODO(cnpryer): Potentially use `ScopeEntry`.
    #[must_use]
    pub fn get_scope_entry<T: AsRef<Path>>(&self, key: T) -> Option<(T, String)> {
        let k = format!("{}", key.as_ref().display());

        // TODO(cnpryer): Smarter escape
        self.doc()
            .get("scopes")
            .and_then(|it| it.get(k).map(|v| (key, escape_string(&v.to_string()))))
    }

    pub fn save<T: AsRef<Path>>(&self, to: T) -> Result<(), Error> {
        write_settings_file(self.doc(), to)
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

pub fn escape_string(s: &str) -> String {
    s.trim().replace(['\\', '"'], "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scopes() {
        let mut db = SettingsDb::new();

        db.insert_scope("/", "default");

        let (_, value) = db.get_scope_entry("/").unwrap();

        assert_eq!(value.to_string(), toml_edit::value("default").to_string());

        db.remove_scope("/");

        let table = db.doc().get("scopes").unwrap();

        assert!(table
            .as_inline_table()
            .map_or(false, toml_edit::InlineTable::is_empty));
    }
}
