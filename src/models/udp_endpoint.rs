use crate::models::udp_endpoint_identity::UdpEndpointIdentity;
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct UdpEndpoint {
    local_addr: IpAddr,
    local_port: u16,
    pid: u32
}

impl UdpEndpoint {
    pub fn new(
            local_addr: IpAddr,
            local_port: u16,
            pid: u32,) -> UdpEndpoint {
                UdpEndpoint {
                    local_addr,
                    local_port,
                    pid,
                }
            }
    
    pub fn identity(&self) -> UdpEndpointIdentity {
        UdpEndpointIdentity {
            local_addr: self.local_addr,
            local_port: self.local_port,
        }
    }
}