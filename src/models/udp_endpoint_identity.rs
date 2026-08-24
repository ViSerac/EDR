use std::net::IpAddr;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct UdpEndpointIdentity {
    pub local_addr: IpAddr,
    pub local_port: u16,
}