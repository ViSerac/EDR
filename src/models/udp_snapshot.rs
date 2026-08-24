use std::collections::HashMap;
use crate::models::udp_endpoint::UdpEndpoint;
use crate::models::udp_endpoint_identity::UdpEndpointIdentity;
use crate::core::differ::snapshot_trait::Snapshot;

pub struct UdpSnapshot {
    endpoints: HashMap<UdpEndpointIdentity, UdpEndpoint>
}

impl UdpSnapshot {
    pub fn new(networks: Vec<UdpEndpoint>) -> UdpSnapshot {
        let mut endpoints_map = HashMap::new();

        for endpoint in networks {
            endpoints_map.insert(endpoint.identity(), endpoint);
        }

        Self {endpoints: endpoints_map}
    }

}

impl Snapshot for UdpSnapshot {
    type Identity = UdpEndpointIdentity;
    type Item = UdpEndpoint;

    fn identities(&self) -> Vec<UdpEndpointIdentity> {
        self.endpoints.keys().cloned().collect()
    }

    fn get(&self, identity: &UdpEndpointIdentity) -> Option<&UdpEndpoint> {
        self.endpoints.get(identity)
    }
}