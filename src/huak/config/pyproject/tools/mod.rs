use self::huak::Huak;
use serde_derive::{Deserialize, Serialize};
pub(crate) mod huak;

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct Tool {
    pub(crate) huak: Huak,
}
