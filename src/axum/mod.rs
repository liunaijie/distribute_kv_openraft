use axum::{
    Json, Router,
    extract::Request,
    http::{self, HeaderName, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::info_span;

mod api;
mod raft;

const REQUEST_ID_HEADER: &str = "x-request-id";

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: bool,
    pub message: Option<String>,
    pub data: Option<Value>,
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl ApiResponse {
    pub fn success_with_empty_data() -> Self {
        Self {
            status: true,
            message: Some("success".to_string()),
            data: None,
        }
    }

    #[allow(dead_code)]
    pub fn success(data: impl Into<Option<Value>>) -> Self {
        Self {
            status: true,
            message: None,
            data: data.into(),
        }
    }

    #[allow(dead_code)]
    pub fn success_with_msg(data: impl Into<Option<Value>>, message: impl Into<String>) -> Self {
        Self {
            status: true,
            message: Some(message.into()),
            data: data.into(),
        }
    }

    pub fn success_with_data<T: Serialize>(data: T) -> Self {
        let data_value = serde_json::to_value(data).unwrap_or(Value::Null);
        Self {
            status: true,
            message: None,
            data: Some(data_value),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: false,
            message: Some(message.into()),
            data: None,
        }
    }
}

pub async fn start_axum_server(port: u32) -> Result<(), anyhow::Error> {
    let app = create_router();
    let address = format!("0.0.0.0:{}", port);
    tracing::info!("Starting server at: http://{}", address);

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    Ok(axum::serve(listener, app).await?)
}
/// 创建路由
fn create_router() -> Router {
    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);

    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let trace_id = request.headers().get(REQUEST_ID_HEADER);
                    match trace_id {
                        Some(trace_id) => info_span!(
                            "",
                            trace_id = ?trace_id,
                        ),
                        None => {
                            tracing::error!("无法提取请求ID");
                            info_span!("")
                        }
                    }
                })
                .on_request(())
                .on_response(()),
        )
        .layer(PropagateRequestIdLayer::new(x_request_id));

    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("*"))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([http::header::CONTENT_TYPE]);

    // let static_service = ServeDir::new("static")
    //     .append_index_html_on_directories(true)
    //     .not_found_service(ServeFile::new("static/index.html"));
    Router::new()
        .route("/health", get(health_check))
        .nest("/api", api::app_routes())
        .nest("/raft", raft::raft_routes())
        .layer(middleware)
        .layer(cors)
}

async fn health_check() -> ApiResponse {
    ApiResponse::success_with_empty_data()
}
