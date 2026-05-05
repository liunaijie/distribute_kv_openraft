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

pub fn v1_routes() -> Router {
    Router::new()
        .route("/set", get(set))
        .route("/get/{key}", get(get_value))
}

#[derive(Deserialize)]
struct SetParams {
    key: String,
    value: String,
}

pub async fn set(Query(params): Query<SetParams>) -> ApiResponse {
    let data_entity = DataEntity::new(params.key, params.value);
    storage::get_storage()
        .as_ref()
        .set(data_entity.clone())
        .await
        .unwrap();
    ApiResponse::success_with_data(data_entity)
}

pub async fn get_value(Path(key): Path<String>) -> ApiResponse {
    let entity = storage::get_storage().as_ref().get(&key).await.unwrap();
    ApiResponse::success_with_data(entity)
}
