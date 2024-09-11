use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetPerformanceResponse {
    pub audio: f32,
    pub total: f32,
}
