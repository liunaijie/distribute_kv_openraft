use std::future::Future;

use crate::storage::{
    multiple_node::openraft_setting::type_config::TypeConfig, type_config::FullSnapshotRequest,
};
use openraft::{
    OptionalSend, RaftNetworkV2, Snapshot, Vote,
    error::{RPCError, ReplicationClosed, StreamingError, Unreachable},
    network::RPCOption,
    raft::{
        AppendEntriesRequest, AppendEntriesResponse, SnapshotResponse, VoteRequest, VoteResponse,
    },
};
use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};

pub struct NetworkConnection {
    rpc_addr: String,
    rpc_port: u32,
    client: Client,
}

impl NetworkConnection {
    pub fn new(rpc_addr: String, rpc_port: u32) -> Self {
        let client = Client::builder().no_proxy().build().unwrap();
        NetworkConnection {
            rpc_addr,
            rpc_port,
            client,
        }
    }

    pub async fn request<Req, Resp>(
        &mut self,
        uri: &str,
        req: Req,
    ) -> Result<Resp, Unreachable<TypeConfig>>
    where
        Req: Serialize + 'static,
        Resp: DeserializeOwned,
    {
        let url = format!("http://{}:{}/{}", self.rpc_addr, self.rpc_port, uri);
        let resp = self
            .client
            .post(url)
            .json(&req)
            .send()
            .await
            .map_err(|e| Unreachable::new(&e))?;

        let body: Resp = resp.json().await.map_err(|e| Unreachable::new(&e))?;
        Ok(body)
    }
}

impl RaftNetworkV2<TypeConfig> for NetworkConnection {
    async fn append_entries(
        &mut self,
        rpc: AppendEntriesRequest<TypeConfig>,
        _option: RPCOption,
    ) -> Result<AppendEntriesResponse<TypeConfig>, RPCError<TypeConfig>> {
        self.request("raft/api/append", rpc)
            .await
            .map_err(RPCError::Unreachable)
    }

    async fn vote(
        &mut self,
        rpc: VoteRequest<TypeConfig>,
        _option: RPCOption,
    ) -> Result<VoteResponse<TypeConfig>, RPCError<TypeConfig>> {
        self.request("raft/api/vote", rpc)
            .await
            .map_err(RPCError::Unreachable)
    }

    async fn full_snapshot(
        &mut self,
        vote: Vote<TypeConfig>,
        snapshot: Snapshot<TypeConfig>,
        _cancel: impl Future<Output = ReplicationClosed> + OptionalSend + 'static,
        _option: RPCOption,
    ) -> Result<SnapshotResponse<TypeConfig>, StreamingError<TypeConfig>> {
        let data: Vec<u8> = snapshot.snapshot.into_inner();
        let req = FullSnapshotRequest {
            vote,
            meta: snapshot.meta,
            data,
        };
        self.request("raft/api/snapshot", req)
            .await
            .map_err(StreamingError::Unreachable)
    }
}
