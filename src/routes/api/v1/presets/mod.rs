use crate::managers::cache::CacheManager;
use crate::managers::database::models::Preset;
use crate::managers::database::DatabaseManager;
use crate::managers::dsp::DSPManager;
use crate::processors::frontline::create_processor_from_type;
use crate::processors::frontline::models::ParameterValue;
use crate::routes::api::v1::presets::models::requests::SaveCurrentPresetRequest;
use crate::routes::api::v1::presets::models::responses::{
    CreateNewPresetResponse, GetCurrentPresetResponse, GetDefaultPresetIdResponse,
    GetPresetsResponse, LoadPresetResponse, PresetBasicInfo,
};
use axum::extract::Path;
use axum::routing::{delete, get, post};
use axum::{Extension, Json, Router};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

mod models;

pub fn create_router() -> Router {
    Router::new()
        .route("/default", get(get_default_preset_id))
        .route("/default", post(set_default_preset_id))
        .route("/", get(get_presets))
        .route("/", post(create_new_preset))
        .route("/current", get(get_current_preset))
        .route("/current", post(save_current_preset))
        .route("/current", delete(delete_current_preset))
        .route("/<preset_id>/load", post(load_preset))
}

async fn get_default_preset_id(
    database_manager: Extension<DatabaseManager>,
) -> Json<GetDefaultPresetIdResponse> {
    Json(GetDefaultPresetIdResponse {
        id: database_manager.get_default_preset_id().await,
    })
}

async fn set_default_preset_id(
    cache_manager: Extension<Arc<Mutex<CacheManager>>>,
    database_manager: Extension<DatabaseManager>,
) {
    let current_preset_id = { cache_manager.lock().unwrap().current_preset_id };
    if let Some(current_preset_id) = current_preset_id {
        let _ = database_manager
            .set_default_preset_id(current_preset_id)
            .await;
    }
}

async fn create_new_preset(
    dsp_manager: Extension<Arc<Mutex<DSPManager>>>,
    cache_manager: Extension<Arc<Mutex<CacheManager>>>,
) -> Json<CreateNewPresetResponse> {
    let id = reset_current_preset(dsp_manager, cache_manager);

    Json(CreateNewPresetResponse { id })
}

async fn get_presets(database_manager: Extension<DatabaseManager>) -> Json<GetPresetsResponse> {
    Json(GetPresetsResponse {
        presets: database_manager
            .get_presets_list()
            .await
            .iter()
            .map(|preset| PresetBasicInfo {
                id: preset.id,
                name: preset.name.clone(),
            })
            .collect(),
    })
}

async fn get_current_preset(
    dsp_manager: Extension<Arc<Mutex<DSPManager>>>,
    database_manager: Extension<DatabaseManager>,
    cache_manager: Extension<Arc<Mutex<CacheManager>>>,
) -> Json<GetCurrentPresetResponse> {
    let current_preset =
        get_current_preset_info(dsp_manager, database_manager, cache_manager).await;
    Json(GetCurrentPresetResponse {
        preset: current_preset,
    })
}

async fn save_current_preset(
    dsp_manager: Extension<Arc<Mutex<DSPManager>>>,
    database_manager: Extension<DatabaseManager>,
    cache_manager: Extension<Arc<Mutex<CacheManager>>>,
    Json(payload): Json<SaveCurrentPresetRequest>,
) {
    let current_preset_id = { cache_manager.lock().unwrap().current_preset_id };
    if let Some(current_preset_id) = current_preset_id {
        let current_preset =
            get_current_preset_info(dsp_manager, database_manager.clone(), cache_manager).await;
        if let Some(mut current_preset) = current_preset {
            current_preset.name = payload.name;
            database_manager.delete_preset(current_preset_id).await;
            database_manager.save_preset(current_preset).await;
        }
    }
}

async fn delete_current_preset(
    dsp_manager: Extension<Arc<Mutex<DSPManager>>>,
    database_manager: Extension<DatabaseManager>,
    cache_manager: Extension<Arc<Mutex<CacheManager>>>,
) {
    let current_preset_id = { cache_manager.lock().unwrap().current_preset_id };
    let default_preset_id = database_manager.get_default_preset_id().await;
    if let Some(current_preset_id) = current_preset_id {
        database_manager.delete_preset(current_preset_id).await;
        if let (Some(first_preset), Some(default_preset_id)) = (
            database_manager.get_presets_list().await.first(),
            default_preset_id,
        ) {
            if default_preset_id == current_preset_id {
                let _ = database_manager
                    .set_default_preset_id(first_preset.id)
                    .await;
            }
        }
    }

    let _ = reset_current_preset(dsp_manager, cache_manager);
}

async fn load_preset(
    Path(preset_id): Path<Uuid>,
    dsp_manager: Extension<Arc<Mutex<DSPManager>>>,
    database_manager: Extension<DatabaseManager>,
    cache_manager: Extension<Arc<Mutex<CacheManager>>>,
) -> Json<LoadPresetResponse> {
    if let Some(preset) = database_manager.get_preset(preset_id).await {
        let mut dsp_manager = dsp_manager.lock().unwrap();
        cache_manager.lock().unwrap().current_preset_id = Some(preset_id);

        dsp_manager.clear_all_processors();
        preset
            .processors
            .iter()
            .enumerate()
            .for_each(|(index, processor_info)| {
                let mut processor = create_processor_from_type(
                    &processor_info.processor_type,
                    &dsp_manager.sample_rate,
                    &dsp_manager.buffer_size,
                );
                processor_info
                    .parameters
                    .numeric
                    .iter()
                    .for_each(|(parameter, value)| {
                        processor.set_parameter(*parameter, ParameterValue::Numeric(*value))
                    });
                processor_info
                    .parameters
                    .string
                    .iter()
                    .for_each(|(parameter, value)| {
                        processor.set_parameter(*parameter, ParameterValue::String(value.clone()))
                    });
                dsp_manager.add_processor(index, processor);
                let _ = dsp_manager.transform_processor_settings(index, |settings| {
                    settings.bypassed = processor_info.settings.bypassed;
                });
            });

        Json(Some(preset))
    } else {
        Json(None)
    }
}

fn reset_current_preset(
    dsp_manager: Extension<Arc<Mutex<DSPManager>>>,
    cache_manager: Extension<Arc<Mutex<CacheManager>>>,
) -> Uuid {
    let id = Uuid::new_v4();
    cache_manager.lock().unwrap().current_preset_id = Some(id);
    dsp_manager.lock().unwrap().clear_all_processors();

    id
}

async fn get_current_preset_info(
    dsp_manager: Extension<Arc<Mutex<DSPManager>>>,
    database_manager: Extension<DatabaseManager>,
    cache_manager: Extension<Arc<Mutex<CacheManager>>>,
) -> Option<Preset> {
    let current_preset_id = { cache_manager.lock().unwrap().current_preset_id };
    let preset_from_db = if let Some(id) = current_preset_id {
        database_manager.get_preset(id).await
    } else {
        None
    };
    preset_from_db
        .or(current_preset_id.map(|id| Preset {
            id,
            is_default: false,
            name: "".to_string(),
            processors: vec![],
        }))
        .map(|mut preset| {
            preset.processors = dsp_manager.lock().unwrap().get_processors_info();
            preset
        })
}
