use std::collections::HashMap;
use crate::models::tcp_connection::TcpConnection;
use crate::models::tcp_connection_identity::TcpConnectionIdentity;
use crate::core::differ::snapshot_trait::Snapshot;

pub struct TcpSnapshot {
    networks: HashMap<TcpConnectionIdentity, TcpConnection>
}

impl TcpSnapshot {
    pub fn new(networks: Vec<TcpConnection>) -> TcpSnapshot {
        let mut networks_map = HashMap::new();

        for network in networks {
            networks_map.insert(network.identity(), network);
        }

        Self {networks: networks_map}
    }

}

impl Snapshot for TcpSnapshot {
    type Identity = TcpConnectionIdentity;
    type Item = TcpConnection;

    fn identities(&self) -> Vec<TcpConnectionIdentity> {
        self.networks.keys().cloned().collect()
    }

    fn get(&self, identity: &TcpConnectionIdentity) -> Option<&TcpConnection> {
        self.networks.get(identity)
    }
}