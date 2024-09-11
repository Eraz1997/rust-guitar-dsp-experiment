use axum::Router;

mod device;
mod healthy;
mod performance;
mod presets;
mod processors;

pub fn create_router() -> Router {
    // TODO: add "/capture" and "/processing" routes
    Router::new()
        .nest("/healthy", healthy::create_router())
        .nest("/performance", performance::create_router())
        .nest("/presets", presets::create_router())
        .nest("/processors", processors::create_router())
        .nest("/device", device::create_router())
}
