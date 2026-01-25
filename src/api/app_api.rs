use axum::{
    Router,
    routing::{get, post},
};

pub fn app_routes() -> Router {
    Router::new().nest("/v1", v1_routes())
}

fn v1_routes() -> Router {
    Router::new()
        .route("/set", post(|| async { "Hello, world!" }))
        .route("/get", get(|| async { "Hello, world!" }))
}
