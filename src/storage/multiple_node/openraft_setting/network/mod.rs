mod connection;
pub mod proxy;

use crate::storage::multiple_node::openraft_setting::network::connection::NetworkConnection;
use crate::storage::multiple_node::openraft_setting::type_config::{
    AppNode, AppNodeId, TypeConfig,
};
use openraft::RaftNetworkFactory;

pub struct Network {}

impl Network {
    pub fn new() -> Network {
        Network {}
    }
}

impl RaftNetworkFactory<TypeConfig> for Network {
    type Network = NetworkConnection;

    async fn new_client(&mut self, _target: AppNodeId, node: &AppNode) -> Self::Network {
        let addr = node.rpc_addr.to_string();
        let port = node.rpc_port;
        NetworkConnection::new(addr, port)
    }
}
