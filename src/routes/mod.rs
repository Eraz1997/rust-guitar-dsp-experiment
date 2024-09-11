use axum::Router;

pub mod api;

pub fn create_router() -> Router {
    Router::new().nest("/api", api::create_router())
}
