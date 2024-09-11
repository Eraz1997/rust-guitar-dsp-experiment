use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SaveCurrentPresetRequest {
    pub name: String,
}
