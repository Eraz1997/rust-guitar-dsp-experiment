use axum::routing::get;
use axum::Router;

pub fn create_router() -> Router {
    Router::new().route("/", get(healthy))
}

async fn healthy() -> &'static str {
    "OK"
}
