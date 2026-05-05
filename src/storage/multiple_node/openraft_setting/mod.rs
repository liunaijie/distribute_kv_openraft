pub mod manager;
mod network;
mod store;
pub mod type_config;

use crate::storage::entity::DataEntity;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RaftInnerRequest {
    Set { key: String, value: DataEntity },
}

impl RaftInnerRequest {
    pub fn set(key: String, value: DataEntity) -> Self {
        Self::Set { key, value }
    }
}

impl Display for RaftInnerRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RaftInnerRequest::Set { key, value } => {
                write!(f, "Set {{ key: {}, value: {} }}", key, value)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RaftInnerResponse {
    pub value: Option<DataEntity>,
}

impl RaftInnerResponse {
    pub fn new(value: DataEntity) -> Self {
        RaftInnerResponse { value: Some(value) }
    }

    pub fn none() -> Self {
        RaftInnerResponse { value: None }
    }
}
