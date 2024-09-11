use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetSettingsResponse {
    pub input_gain: f32,
    pub volume: f32,
    pub is_mic_bias_on: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SetInputGainRequest {
    pub value: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SetVolumeRequest {
    pub value: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SetMicBiasRequest {
    pub on: bool,
}
