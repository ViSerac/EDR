use std::time::SystemTime;
use std::path::PathBuf;

use super::process::Process;
use super::tcp_connection::TcpConnection;
use super::udp_endpoint::UdpEndpoint;

#[derive(Debug, Clone)]
pub enum Event {
    ProcessCreated(Process, SystemTime),
    ProcessTerminated(Process, SystemTime),
    ConnectionOpened(TcpConnection, SystemTime),
    ConnectionClosed(TcpConnection, SystemTime),
    UdpEndpointOpened(UdpEndpoint, SystemTime),
    UdpEndpointClosed(UdpEndpoint, SystemTime),
    FileCreated(PathBuf, SystemTime),
    FileModified(PathBuf, SystemTime),
    FileRemoved(PathBuf, SystemTime),
    FileRenamed(PathBuf, PathBuf, SystemTime),
}