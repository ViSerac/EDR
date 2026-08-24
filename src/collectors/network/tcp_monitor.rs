use crate::models::{tcp_snapshot::TcpSnapshot, tcp_connection::TcpConnection, event::Event};
use crate::core::differ::snapshot_differ::{diff, build_tcp_events};

pub struct TcpMonitor {
    last_snapshot: Option<TcpSnapshot>,
}

impl TcpMonitor {
    pub fn new() -> TcpMonitor {
        Self {last_snapshot: None}
    }

    pub fn run_cycle(&mut self, current_networks: Vec<TcpConnection>) -> Vec<Event> {
        let new_snapshot = TcpSnapshot::new(current_networks);

        let events = match self.last_snapshot.take() {
            Some(old_snapshot) => {
                let diff_result = diff(&old_snapshot, &new_snapshot);
                build_tcp_events(diff_result, &old_snapshot, &new_snapshot)
            }
            None => Vec::new()
        };

        self.last_snapshot = Some(new_snapshot);

        events
    }
}
