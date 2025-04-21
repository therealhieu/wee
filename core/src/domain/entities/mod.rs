use serde::{Deserialize, Serialize};

pub mod url;

pub trait Entity: Serialize + for<'a> Deserialize<'a> {
    fn to_json(&self) -> Result<String, anyhow::Error> {
        serde_json::to_string(self).map_err(anyhow::Error::from)
    }

    fn from_json(json: &str) -> Result<Self, anyhow::Error> {
        serde_json::from_str(json).map_err(anyhow::Error::from)
    }
}
