use crate::collectors::network::raw_udp_data::RawUdpData;
use crate::models::udp_endpoint::UdpEndpoint;

impl From<RawUdpData> for UdpEndpoint {
    fn from(raw: RawUdpData) -> Self {
        UdpEndpoint::new(raw.local_addr, raw.local_port, raw.pid)
    }
}