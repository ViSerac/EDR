use crate::models::{snapshot_diff::SnapshotDiff, event::Event,
    process_snapshot::ProcessSnapshot, process_identity::ProcessIdentity,
    tcp_snapshot::TcpSnapshot, tcp_connection_identity::TcpConnectionIdentity,
    udp_snapshot::UdpSnapshot, udp_endpoint_identity::UdpEndpointIdentity};
use crate::core::differ::snapshot_trait::Snapshot;
use std::time::SystemTime;

pub fn diff<T: Snapshot>(old: &T, new: &T) -> SnapshotDiff<T::Identity> {
    let mut created = Vec::new();

    for identity in new.identities() {
        if old.get(&identity).is_none() {
            created.push(identity);
        }
    }

    let mut terminated = Vec::new();

    for identity in old.identities() {
        if new.get(&identity).is_none() {
            terminated.push(identity);
        }
    }

    SnapshotDiff {created, terminated}
}

pub fn build_process_events(diff: SnapshotDiff<ProcessIdentity>, old: &ProcessSnapshot, new: &ProcessSnapshot) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();

    for identity in diff.created {
        match new.get(&identity) {
        Some(process) => {
            let cloned_process = process.clone();
            let event = Event::ProcessCreated(cloned_process, SystemTime::now());
            events.push(event);
        }
        None => {
            eprintln!("Warning: identity in diff.created not found on new snapshot: {:?}", identity);
            }
        }
    }

    for identity in diff.terminated {
        match  old.get(&identity) {
        Some(process) => {
            let cloned_process = process.clone();
            let event = Event::ProcessTerminated(cloned_process, SystemTime::now());
            events.push(event);
        }
        None => {
            eprintln!("Warning: identity in diff.terminated not found on new snapshot: {:?}", identity);
            }           
        }
    }
    events
}

pub fn build_tcp_events(diff: SnapshotDiff<TcpConnectionIdentity>, old: &TcpSnapshot, new: &TcpSnapshot) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();

    for identity in diff.created {
        match new.get(&identity) {
        Some(network) => {
            let cloned_network = network.clone();
            let event = Event::ConnectionOpened(cloned_network, SystemTime::now());
            events.push(event);
        }
        None => {
            eprintln!("Warning: identity in diff.created not found on new snapshot: {:?}", identity);
            }
        }
    }

    for identity in diff.terminated {
        match  old.get(&identity) {
        Some(network) => {
            let cloned_network = network.clone();
            let event = Event::ConnectionClosed(cloned_network, SystemTime::now());
            events.push(event);
        }
        None => {
            eprintln!("Warning: identity in diff.terminated not found on new snapshot: {:?}", identity);
            }           
        }
    }
    events
}

pub fn build_udp_events(diff: SnapshotDiff<UdpEndpointIdentity>, old: &UdpSnapshot, new: &UdpSnapshot) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();

    for identity in diff.created {
        match new.get(&identity) {
        Some(endpoint) => {
            let cloned_endpoint = endpoint.clone();
            let event = Event::UdpEndpointOpened(cloned_endpoint, SystemTime::now());
            events.push(event);
        }
        None => {
            eprintln!("Warning: identity in diff.created not found on new snapshot: {:?}", identity);
            }
        }
    }

    for identity in diff.terminated {
        match  old.get(&identity) {
        Some(endpoint) => {
            let cloned_endpoint = endpoint.clone();
            let event = Event::UdpEndpointClosed(cloned_endpoint, SystemTime::now());
            events.push(event);
        }
        None => {
            eprintln!("Warning: identity in diff.terminated not found on new snapshot: {:?}", identity);
            }           
        }
    }
    events
}