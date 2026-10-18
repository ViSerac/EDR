use std::env;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui;

use crate::collectors::file::file_monitor::FileMonitor;
use crate::collectors::file::watched_folder::start_file_watchers;
use crate::collectors::file::watched_path::WatchedPath;
use crate::collectors::network::{tcp_monitor::TcpMonitor, tcp_capture::capture_tcp_v4, tcp_capture::capture_tcp_v6};
use crate::collectors::network::{udp_monitor::UdpMonitor, udp_capture::capture_udp_v4, udp_capture::capture_udp_v6};
use crate::collectors::process::process_capture::capture_raw_processes;
use crate::collectors::process::process_monitor::ProcessMonitor;
use crate::core::rules::rule_engine::RuleEngine;
use crate::models::alert::Alert;
use crate::models::event::Event;
use crate::models::process::Process;
use crate::models::tcp_connection::TcpConnection;
use crate::models::udp_endpoint::UdpEndpoint;
use crate::rules::file_then_process_rule::FileThenProcessRule;
use crate::rules::suspicious_parent_rule::SuspiciousParentRule;

const CAPTURE_INTERVAL: Duration = Duration::from_millis(500);

/// The UI side of the channels has been dropped; the capture loop should stop.
struct UiDisconnected;

pub fn spawn_capture_thread(event_tx: Sender<Event>, alert_tx: Sender<Alert>, ctx: egui::Context) {
    thread::spawn(move || run_capture_loop(event_tx, alert_tx, ctx));
}

fn capture_all_tcp() -> Vec<TcpConnection> {
    let mut raw_data = capture_tcp_v4();
    raw_data.extend(capture_tcp_v6());
    raw_data.into_iter().map(TcpConnection::from).collect()
}

fn capture_all_udp() -> Vec<UdpEndpoint> {
    let mut raw_data = capture_udp_v4();
    raw_data.extend(capture_udp_v6());
    raw_data.into_iter().map(UdpEndpoint::from).collect()
}

fn dispatch(
    events: Vec<Event>,
    rule_engine: &mut RuleEngine,
    event_tx: &Sender<Event>,
    alert_tx: &Sender<Alert>,
) -> Result<bool, UiDisconnected> {
    let mut sent_any = false;

    for event in events {
        let alerts = rule_engine.process_event(&event);

        event_tx.send(event).map_err(|_| UiDisconnected)?;
        sent_any = true;

        for alert in alerts {
            alert_tx.send(alert).map_err(|_| UiDisconnected)?;
        }
    }

    Ok(sent_any)
}

fn run_capture_loop(event_tx: Sender<Event>, alert_tx: Sender<Alert>, ctx: egui::Context) {
    let mut process_monitor = ProcessMonitor::new();
    let mut tcp_monitor = TcpMonitor::new();
    let mut udp_monitor = UdpMonitor::new();

    let watched_paths = vec![
        WatchedPath {
            path: env::temp_dir(),
            recursive: false,
            ignored_extensions: vec![],
        },
    ];

    let file_receiver = start_file_watchers(watched_paths);
    let mut file_monitor = FileMonitor::new(file_receiver);

    let mut rule_engine = RuleEngine::new(vec![
        Box::new(SuspiciousParentRule::new()),
        Box::new(FileThenProcessRule::new()),
    ]);

    loop {
        let cycle_start = Instant::now();
        let mut new_data = false;

        let file_events = file_monitor.poll_events();
        match dispatch(file_events, &mut rule_engine, &event_tx, &alert_tx) {
            Ok(sent) => new_data |= sent,
            Err(UiDisconnected) => return,
        }

        match capture_raw_processes() {
            Ok(raw_process) => {
                let mut processes: Vec<Process> = Vec::new();
                for process in raw_process {
                    match Process::try_from(process) {
                        Ok(converted_process) => processes.push(converted_process),
                        Err(process_error) => eprintln!("Error trying to capture {:?}", process_error),
                    }
                }
                let events = process_monitor.run_cycle(processes);
                match dispatch(events, &mut rule_engine, &event_tx, &alert_tx) {
                    Ok(sent) => new_data |= sent,
                    Err(UiDisconnected) => return,
                }
            }
            Err(capture_error) => eprintln!("Error trying to capture {:?}", capture_error),
        }

        let tcp_events = tcp_monitor.run_cycle(capture_all_tcp());
        match dispatch(tcp_events, &mut rule_engine, &event_tx, &alert_tx) {
            Ok(sent) => new_data |= sent,
            Err(UiDisconnected) => return,
        }

        let udp_events = udp_monitor.run_cycle(capture_all_udp());
        match dispatch(udp_events, &mut rule_engine, &event_tx, &alert_tx) {
            Ok(sent) => new_data |= sent,
            Err(UiDisconnected) => return,
        }

        // Wake the UI thread so it drains the channels without waiting for input.
        if new_data {
            ctx.request_repaint();
        }

        let elapsed = cycle_start.elapsed();
        match CAPTURE_INTERVAL.checked_sub(elapsed) {
            Some(remaining) => thread::sleep(remaining),
            None => eprintln!("Capture cycle took {:?}, longer than the {:?} interval", elapsed, CAPTURE_INTERVAL),
        }
    }
}
