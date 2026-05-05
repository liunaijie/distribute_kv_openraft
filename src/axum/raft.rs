use std::io::Cursor;

use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use openraft::{
    Snapshot,
    raft::{
        AppendEntriesRequest, AppendEntriesResponse, SnapshotResponse, TransferLeaderRequest,
        VoteRequest, VoteResponse,
    },
};

use crate::storage::{
    OpenRaftManagerService,
    type_config::{FullSnapshotRequest, TypeConfig},
};

pub fn raft_routes() -> Router {
    Router::new()
        .nest("/api", api_routes())
        .nest("/admin", admin_routes())
}

fn api_routes() -> Router {
    Router::new()
        .route("/append", post(append_handler))
        .route("/snapshot", post(snapshot_handler))
        .route("/vote", post(vote_handler))
        .route("/transfer_leader", post(transfer_leader_handler))
}

fn admin_routes() -> Router {
    Router::new()
        .route("/add_learner", get(|| async { "add_learner" }))
        .route("/change_membership", get(|| async { "change_membership" }))
        .route("/init", get(|| async { "init" }))
        .route("/metrics", get(|| async { "metrics" }))
}

async fn append_handler(
    Json(append_request): Json<AppendEntriesRequest<TypeConfig>>,
) -> Result<Json<AppendEntriesResponse<TypeConfig>>, (StatusCode, String)> {
    OpenRaftManagerService::get_raft_instance()
        .append_entries(append_request)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn snapshot_handler(
    Json(snapshot_request): Json<FullSnapshotRequest>,
) -> Result<Json<SnapshotResponse<TypeConfig>>, (StatusCode, String)> {
    let req = Snapshot {
        meta: snapshot_request.meta,
        snapshot: Cursor::new(snapshot_request.data),
    };
    OpenRaftManagerService::get_raft_instance()
        .install_full_snapshot(snapshot_request.vote, req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn vote_handler(
    Json(vote_request): Json<VoteRequest<TypeConfig>>,
) -> Result<Json<VoteResponse<TypeConfig>>, (StatusCode, String)> {
    
    OpenRaftManagerService::get_raft_instance()
        .vote(vote_request)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn transfer_leader_handler(
    Json(transfer_leader_request): Json<TransferLeaderRequest<TypeConfig>>,
) -> Result<(), (StatusCode, String)> {
    OpenRaftManagerService::get_raft_instance()
        .handle_transfer_leader(transfer_leader_request)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
