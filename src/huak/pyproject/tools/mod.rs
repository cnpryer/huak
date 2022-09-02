use serde_derive::{Deserialize, Serialize};

use self::huak::Huak;

pub mod huak;

#[derive(Serialize, Deserialize, Default)]
pub struct Tool {
    pub huak: Huak,
}

impl Tool {
    pub fn new() -> Tool {
        Tool::default()
    }

    pub fn huak(&self) -> &Huak {
        &self.huak
    }

    pub fn set_huak(&mut self, huak: Huak) {
        self.huak = huak
    }
}
