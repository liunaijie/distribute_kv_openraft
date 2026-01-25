use axum::{Router, routing::get};

pub fn admin_routes() -> Router {
    Router::new().route("/login", get(|| async { "Login!" }))
}
