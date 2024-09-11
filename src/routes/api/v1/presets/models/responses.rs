use crate::managers::database::models::Preset;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct GetDefaultPresetIdResponse {
    pub id: Option<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateNewPresetResponse {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct PresetBasicInfo {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetPresetsResponse {
    pub presets: Vec<PresetBasicInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct GetCurrentPresetResponse {
    pub preset: Option<Preset>,
}

pub type LoadPresetResponse = Option<Preset>;
