use axum::Router;

pub mod v1;

pub fn create_router() -> Router {
    Router::new().nest("/v1", v1::create_router())
}
