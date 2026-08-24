use std::net::IpAddr;

pub struct RawUdpData {
    pub local_addr: IpAddr,
    pub local_port: u16,
    pub pid: u32,
}