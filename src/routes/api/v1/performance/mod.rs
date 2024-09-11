use crate::managers::dsp::DSPManager;
use crate::managers::performance::PerformanceManager;
use crate::routes::api::v1::performance::models::GetPerformanceResponse;
use axum::routing::get;
use axum::{Extension, Json, Router};
use std::sync::{Arc, Mutex};

mod models;

pub fn create_router() -> Router {
    Router::new().route("/", get(performance))
}

async fn performance(
    dsp_manager: Extension<Arc<Mutex<DSPManager>>>,
    performance_manager: Extension<Arc<PerformanceManager>>,
) -> Json<GetPerformanceResponse> {
    Json(GetPerformanceResponse {
        audio: dsp_manager
            .lock()
            .unwrap()
            .get_audio_cpu_usage()
            .unwrap_or(0.0),
        total: performance_manager.get_total_cpu_usage(),
    })
}
