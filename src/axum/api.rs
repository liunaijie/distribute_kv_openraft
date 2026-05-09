use axum::{
    Router,
    extract::{Path, Query},
    routing::get,
};
use serde::Deserialize;

use crate::axum::ApiResponse;
use crate::service::APP_STATE;

pub fn app_routes() -> Router {
    Router::new().nest("/v1", v1_routes())
}

fn v1_routes() -> Router {
    Router::new()
        .route("/set", get(set))
        .route("/get/{key}", get(get_value))
        .route("/del/{key}", get(del_value))
}

#[derive(Deserialize)]
struct SetParams {
    key: String,
    value: String,
}

async fn set(Query(params): Query<SetParams>) -> ApiResponse {
    match APP_STATE
        .get()
        .unwrap()
        .storage
        .set(params.key.as_str(), params.value.as_str())
        .await
    {
        Ok(x) => ApiResponse::success_with_data(x),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

#[derive(Deserialize)]
struct GetQueryParams {
    linearize: Option<bool>,
}

async fn get_value(Path(key): Path<String>, Query(params): Query<GetQueryParams>) -> ApiResponse {
    match APP_STATE
        .get()
        .unwrap()
        .storage
        .get(key.as_str(), params.linearize.unwrap_or(false))
        .await
    {
        Ok(entity) => ApiResponse::success_with_data(entity),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

async fn del_value(Path(key): Path<String>) -> ApiResponse {
    match APP_STATE.get().unwrap().storage.delete(key.as_str()).await {
        Ok(_) => ApiResponse::success_with_empty_data(),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}
