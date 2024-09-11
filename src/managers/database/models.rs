use crate::managers::dsp::models::ProcessorInfo;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Preset {
    pub id: Uuid,
    pub is_default: bool,
    pub name: String,
    pub processors: Vec<ProcessorInfo>,
}
