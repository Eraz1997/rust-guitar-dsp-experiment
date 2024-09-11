use crate::managers::audio_device_settings::AudioDeviceSettingsManager;
use crate::routes::api::v1::device::models::{
    GetSettingsResponse, SetInputGainRequest, SetMicBiasRequest, SetVolumeRequest,
};
use axum::routing::{get, put};
use axum::{Extension, Json, Router};
use std::sync::Arc;

mod models;

pub fn create_router() -> Router {
    Router::new()
        .route("/settings", get(get_settings))
        .route("/input-gain", put(set_input_gain))
        .route("/volume", put(set_volume))
        .route("/mic-bias", put(set_mic_bias))
}

async fn get_settings(
    audio_device_settings_manager: Extension<Arc<AudioDeviceSettingsManager>>,
) -> Json<GetSettingsResponse> {
    Json(GetSettingsResponse {
        input_gain: audio_device_settings_manager
            .get_input_gain()
            .unwrap_or(0.0),
        volume: audio_device_settings_manager.get_volume().unwrap_or(0.0),
        is_mic_bias_on: audio_device_settings_manager
            .is_mic_bias_on()
            .unwrap_or(false),
    })
}

async fn set_input_gain(
    audio_device_settings_manager: Extension<Arc<AudioDeviceSettingsManager>>,
    Json(payload): Json<SetInputGainRequest>,
) {
    audio_device_settings_manager
        .set_input_gain(payload.value)
        .unwrap();
}

async fn set_volume(
    audio_device_settings_manager: Extension<Arc<AudioDeviceSettingsManager>>,
    Json(payload): Json<SetVolumeRequest>,
) {
    audio_device_settings_manager
        .set_volume(payload.value)
        .unwrap();
}

async fn set_mic_bias(
    audio_device_settings_manager: Extension<Arc<AudioDeviceSettingsManager>>,
    Json(payload): Json<SetMicBiasRequest>,
) {
    audio_device_settings_manager
        .set_mic_bias(payload.on)
        .unwrap();
}
