use crate::collectors::network::raw_udp_data::RawUdpData;

// Stub until the /proc/net/udp collector exists.
pub fn capture_udp_v4() -> Vec<RawUdpData> {
    Vec::new()
}

// Stub until the /proc/net/udp6 collector exists.
pub fn capture_udp_v6() -> Vec<RawUdpData> {
    Vec::new()
}
