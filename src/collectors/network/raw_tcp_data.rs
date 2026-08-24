use std::net::IpAddr;
use crate::models::tcp_connection_identity::Protocol;

pub struct RawTcpData {
    pub local_addr: IpAddr,
    pub local_port: u16,
    pub remote_addr: IpAddr,
    pub remote_port: u16,
    pub protocol: Protocol,
    pub pid: u32,
    pub state: Option<u32>,
}