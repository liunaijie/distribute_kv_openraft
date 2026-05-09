mod connection;

use openraft::RaftNetworkFactory;

use crate::{
    raft::{
        network::connection::NetworkConnection,
        type_config::{AppNodeId, TypeConfig},
    },
    utils::config::NodeRpcInfo,
};

pub struct Network {}

impl Network {
    pub fn new() -> Network {
        Network {}
    }
}

impl RaftNetworkFactory<TypeConfig> for Network {
    type Network = NetworkConnection;

    async fn new_client(&mut self, _target: AppNodeId, node: &NodeRpcInfo) -> Self::Network {
        let addr = node.rpc_addr.to_string();
        let port = node.rpc_port;
        NetworkConnection::new(addr, port)
    }
}
