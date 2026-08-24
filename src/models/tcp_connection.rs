use crate::models::tcp_connection_identity::TcpConnectionIdentity;
use crate::models::tcp_connection_identity::Protocol;
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct TcpConnection {
    local_addr: IpAddr,
    local_port: u16,
    remote_addr: IpAddr,
    remote_port: u16,
    protocol: Protocol,
    pid: u32,
    state: Option<TcpState>
}

#[derive(Debug, Clone)]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
    DeleteTcb,
    Unknown(u32),
}

impl TcpConnection {
    pub fn new(
        local_addr: IpAddr,
        local_port: u16,
        remote_addr: IpAddr,
        remote_port: u16,
        protocol: Protocol,
        pid: u32,
        state: Option<TcpState>,
    ) -> TcpConnection {
        TcpConnection {
            local_addr,
            local_port,
            remote_addr,
            remote_port,
            protocol,
            pid,
            state,
        }
    }

    pub fn identity(&self) -> TcpConnectionIdentity {
        TcpConnectionIdentity {
            local_ip: self.local_addr,
            local_port: self.local_port,
            remote_ip: self.remote_addr,
            remote_port: self.remote_port,
            protocol: self.protocol,
        }
    }
}