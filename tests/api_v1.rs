use axum::{body::Body, http::Request};
use tower::ServiceExt;

use distribute_kv_openraft::{axum::api::app_routes, storage, utils::config::StorageType};

#[tokio::test]
async fn v1_set_should_not_panic_when_storage_not_initialized() {
    let app = app_routes();

    let req = Request::builder()
        .uri("/v1/set?key=k1&value=v1")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();

    assert_eq!(resp.status().as_u16(), 200);
}

#[tokio::test]
async fn v1_get_should_not_panic_when_storage_not_initialized() {
    storage::init_storage(StorageType::Memory).await;
    let app = app_routes();

    let req = Request::builder()
        .uri("/v1/get/k1")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();

    assert_eq!(resp.status().as_u16(), 200);
}

#[tokio::test]
async fn v1_del_should_not_panic_when_storage_not_initialized() {
    let app = app_routes();

    let req = Request::builder()
        .uri("/v1/del/k1")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();

    assert_eq!(resp.status().as_u16(), 200);
}
