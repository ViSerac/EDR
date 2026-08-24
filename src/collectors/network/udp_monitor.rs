use crate::models::{udp_snapshot::UdpSnapshot, udp_endpoint::UdpEndpoint, event::Event};
use crate::core::differ::snapshot_differ::{diff, build_udp_events};

pub struct UdpMonitor {
    last_snapshot: Option<UdpSnapshot>,
}

impl UdpMonitor {
    pub fn new() -> UdpMonitor {
        Self {last_snapshot: None}
    }

    pub fn run_cycle(&mut self, current_networks: Vec<UdpEndpoint>) -> Vec<Event> {
        let new_snapshot = UdpSnapshot::new(current_networks);

        let events = match self.last_snapshot.take() {
            Some(old_snapshot) => {
                let diff_result = diff(&old_snapshot, &new_snapshot);
                build_udp_events(diff_result, &old_snapshot, &new_snapshot)
            }
            None => Vec::new()
        };

        self.last_snapshot = Some(new_snapshot);

        events
    }
}
