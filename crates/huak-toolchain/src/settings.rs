//! This module implements read and write functionality for Huak's persisted application data.
use crate::Error;
use std::path::Path;
use toml_edit::Document;

#[derive(Default)]
pub struct SettingsDb {
    doc: Document,
}

impl SettingsDb {
    pub fn new(doc: Document) -> Self {
        Self { doc }
    }

    pub fn doc(&self) -> &Document {
        &self.doc
    }

    pub fn doc_mut(&mut self) -> &mut Document {
        &mut self.doc
    }

    pub fn try_from<T: AsRef<Path>>(path: T) -> Result<Self, Error> {
        Ok(SettingsDb::new(read_settings_file(path)?))
    }
}

/// A helper for reading the contents of a settings.toml file.
pub(crate) fn read_settings_file<T: AsRef<Path>>(path: T) -> Result<Document, Error> {
    let doc = std::str::from_utf8(std::fs::read(path)?.as_slice())?.parse::<Document>()?;

    Ok(doc)
}
