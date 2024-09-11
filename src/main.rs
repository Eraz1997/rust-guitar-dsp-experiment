use crate::logger::Logger;
use crate::managers::audio_device_settings::AudioDeviceSettingsManager;
use crate::managers::cache::CacheManager;
use crate::managers::database::DatabaseManager;
use crate::managers::dsp::DSPManager;
use crate::managers::file_system::FileSystemManager;
use crate::managers::performance::PerformanceManager;
use crate::routes::create_router;
use crate::server::Server;
use crate::settings::Settings;
use axum::Extension;
use clap::Parser;
use std::sync::{Arc, Mutex};
use tower_http::trace::TraceLayer;

mod logger;
mod managers;
mod processors;
mod routes;
mod server;
mod settings;

#[tokio::main]
async fn main() {
    let settings = Settings::parse();

    Logger::new(&settings).init();

    let dsp_manager = Arc::new(Mutex::new(DSPManager::new(&settings).unwrap()));
    let performance_manager = Arc::new(PerformanceManager::new());
    let file_system_manager = FileSystemManager::new().unwrap();
    let database_manager = DatabaseManager::new(&settings).await.unwrap();
    let cache_manager = Arc::new(Mutex::new(CacheManager::new()));

    dsp_manager.lock().unwrap().start().unwrap();

    let mut app = create_router()
        .layer(TraceLayer::new_for_http())
        .layer(Extension(dsp_manager))
        .layer(Extension(cache_manager))
        .layer(Extension(database_manager))
        .layer(Extension(file_system_manager))
        .layer(Extension(performance_manager));

    if settings.hifiberry_enabled {
        let audio_device_settings_manager = Arc::new(AudioDeviceSettingsManager::new().unwrap());
        app = app.layer(Extension(audio_device_settings_manager));
    }

    Server::new(&settings).start(&app).await.unwrap();
}
