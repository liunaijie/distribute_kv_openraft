use anyhow::{Error, Result};
use axum::{
    Router,
    extract::Request,
    http::{self, HeaderName, HeaderValue, Method},
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::{info, info_span};

mod admin_api;
mod app_api;

const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn start_http_server(port: u16) -> Result<(), Error> {
    info!("Starting HTTP server...");

    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);

    let tracing = ServiceBuilder::new()
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

    let router = Router::new()
        .nest("/api/app", app_api::app_routes())
        .nest("/api/admin", admin_api::admin_routes())
        .layer(tracing)
        .layer(cors);

    let address = format!("0.0.0.0:{}", port);
    info!("Starting server at: http://{}", address);

    let listener = TcpListener::bind(address)
        .await
        .map_err(|e| Error::msg(format!("Failed to bind address: {}", e)))?;

    axum::serve(listener, router)
        .await
        .map_err(|e| Error::msg(format!("Failed to serve: {}", e)))?;

    Ok(())
}
