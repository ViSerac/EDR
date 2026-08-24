use std::net::IpAddr;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct TcpConnectionIdentity {
    pub local_ip: IpAddr,
    pub local_port: u16,
    pub remote_ip: IpAddr,
    pub remote_port: u16,
    pub protocol: Protocol,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Protocol {
    Tcp,
    Udp,
}