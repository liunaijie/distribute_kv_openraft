use crate::storage::multiple_node::openraft_setting::{RaftInnerRequest, RaftInnerResponse};
use openraft::{SnapshotMeta, Vote};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, io::Cursor};

pub type SnapshotData = Cursor<Vec<u8>>;

openraft::declare_raft_types!(
    pub TypeConfig:
        D = RaftInnerRequest,
        R = RaftInnerResponse,
        NodeId = AppNodeId,
        Node = AppNode,
        Entry = openraft::Entry<TypeConfig>,
        SnapshotData = SnapshotData
);

pub type AppNodeId = u16;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub struct AppNode {
    pub node_id: u16,
    pub rpc_addr: String,
    pub rpc_port: u32,
}

impl Display for AppNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node {{ rpc_addr: {}, node_id: {} }}",
            self.rpc_addr, self.node_id
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullSnapshotRequest {
    pub vote: Vote<TypeConfig>,
    pub meta: SnapshotMeta<TypeConfig>,
    pub data: Vec<u8>,
}
