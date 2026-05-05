use axum::{
    Router,
    extract::{Path, Query},
    routing::get,
};
use serde::Deserialize;

use crate::{
    axum::ApiResponse,
    storage::{self, entity::DataEntity},
};

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
    let data_entity = DataEntity::new(params.key, params.value);
    match storage::get_storage().set(data_entity.clone()).await {
        Ok(_) => ApiResponse::success_with_data(data_entity),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

async fn get_value(Path(key): Path<String>) -> ApiResponse {
    match storage::get_storage().get(key.as_str()).await {
        Ok(entity) => ApiResponse::success_with_data(entity),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

async fn del_value(Path(key): Path<String>) -> ApiResponse {
    match storage::get_storage().delete(key.as_str()).await {
        Ok(_) => ApiResponse::success_with_empty_data(),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}
