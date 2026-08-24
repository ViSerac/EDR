use crate::collectors::network::raw_tcp_data::RawTcpData;
use crate::models::tcp_connection::{TcpConnection, TcpState};

impl From<RawTcpData> for TcpConnection {
    fn from(raw: RawTcpData) -> Self {
        let state = match raw.state {
            None => None,
            Some(code) => {
                let translated = match code {
                    1 => TcpState::Closed,
                    2 => TcpState::Listen,
                    3 => TcpState::SynSent,
                    4 => TcpState::SynReceived,
                    5 => TcpState::Established,
                    6 => TcpState::FinWait1,
                    7 => TcpState::FinWait2,
                    8 => TcpState::CloseWait,
                    9 => TcpState::Closing,
                    10 => TcpState::LastAck,
                    11 => TcpState::TimeWait,
                    12 => TcpState::DeleteTcb,
                    other => TcpState::Unknown(other),
                };
                Some(translated)
            }
        };

        TcpConnection::new(raw.local_addr,
            raw.local_port,
            raw.remote_addr,
            raw.remote_port,
            raw.protocol,
            raw.pid,
            state)
    }
}