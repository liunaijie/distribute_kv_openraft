use crate::utils::config::NodeRpcInfo;
use openraft::{SnapshotMeta, Vote};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    io::Cursor,
};

pub type SnapshotData = Cursor<Vec<u8>>;

openraft::declare_raft_types!(
    pub TypeConfig:
        D = RaftRpcRequest,
        R = RaftRpcResponse,
        NodeId = AppNodeId,
        Node = NodeRpcInfo,
        Entry = openraft::Entry<TypeConfig>,
        SnapshotData = SnapshotData
);

pub type AppNodeId = u16;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullSnapshotRequest {
    pub vote: Vote<TypeConfig>,
    pub meta: SnapshotMeta<TypeConfig>,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RaftRpcRequest {
    pub data_type: RaftRpcRequestType,
    pub value: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RaftRpcRequestType {
    // KV
    KvSet,
    KvDelete,
}

impl RaftRpcRequest {
    pub fn new(data_type: RaftRpcRequestType, value: Vec<u8>) -> RaftRpcRequest {
        RaftRpcRequest { data_type, value }
    }
}

impl Display for RaftRpcRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.data_type, self.value)
    }
}

impl fmt::Display for RaftRpcRequestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RaftRpcRequestType::KvSet => write!(f, "KvSet"),
            RaftRpcRequestType::KvDelete => write!(f, "KvDelete"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RaftRpcResponse {
    pub value: Option<Vec<u8>>,
}

impl RaftRpcResponse {
    pub fn none() -> Self {
        RaftRpcResponse { value: None }
    }

    pub fn new(data: Vec<u8>) -> Self {
        RaftRpcResponse { value: Some(data) }
    }
}
