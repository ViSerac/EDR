use crate::collectors::network::raw_tcp_data::RawTcpData;

// Stub until the /proc/net/tcp collector exists.
pub fn capture_tcp_v4() -> Vec<RawTcpData> {
    Vec::new()
}

// Stub until the /proc/net/tcp6 collector exists.
pub fn capture_tcp_v6() -> Vec<RawTcpData> {
    Vec::new()
}
