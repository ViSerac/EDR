use std::time::SystemTime;

use chrono::{DateTime, Local};
use eframe::egui::Color32;

use crate::models::event::Event;
use crate::models::severity::Severity;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventCategory {
    Process,
    Network,
    File,
}

/// A pre-formatted event, computed once when the event arrives so the feed
/// doesn't re-format thousands of rows every frame.
pub struct EventRow {
    pub time: String,
    pub category: EventCategory,
    pub tag: &'static str,
    pub description: String,
    /// Lowercased text used by the search filter.
    pub search_text: String,
}

impl EventRow {
    pub fn from_event(event: &Event) -> EventRow {
        let (time, category, tag) = match event {
            Event::ProcessCreated(_, t) | Event::ProcessTerminated(_, t) => {
                (*t, EventCategory::Process, "PROC")
            }
            Event::ConnectionOpened(_, t) | Event::ConnectionClosed(_, t) => {
                (*t, EventCategory::Network, "TCP")
            }
            Event::UdpEndpointOpened(_, t) | Event::UdpEndpointClosed(_, t) => {
                (*t, EventCategory::Network, "UDP")
            }
            Event::FileCreated(_, t)
            | Event::FileModified(_, t)
            | Event::FileRemoved(_, t)
            | Event::FileRenamed(_, _, t) => (*t, EventCategory::File, "FILE"),
        };

        let description = describe_event(event);
        let search_text = format!("{} {}", tag, description).to_lowercase();

        EventRow {
            time: format_time(time),
            category,
            tag,
            description,
            search_text,
        }
    }
}

pub fn describe_event(event: &Event) -> String {
    match event {
        Event::ProcessCreated(process, _) => {
            let mut text = format!(
                "Process created: {} (pid {}, parent pid {})",
                process.name(),
                process.pid(),
                process.parent_pid()
            );
            if let Some(path) = process.path() {
                text.push_str(&format!(" - {}", path));
            }
            text
        }
        Event::ProcessTerminated(process, _) => format!(
            "Process terminated: {} (pid {})",
            process.name(),
            process.pid()
        ),
        Event::ConnectionOpened(connection, _) => {
            let id = connection.identity();
            format!(
                "Connection opened: {}:{} -> {}:{}",
                id.local_ip, id.local_port, id.remote_ip, id.remote_port
            )
        }
        Event::ConnectionClosed(connection, _) => {
            let id = connection.identity();
            format!(
                "Connection closed: {}:{} -> {}:{}",
                id.local_ip, id.local_port, id.remote_ip, id.remote_port
            )
        }
        Event::UdpEndpointOpened(endpoint, _) => {
            let id = endpoint.identity();
            format!("UDP endpoint opened: {}:{}", id.local_addr, id.local_port)
        }
        Event::UdpEndpointClosed(endpoint, _) => {
            let id = endpoint.identity();
            format!("UDP endpoint closed: {}:{}", id.local_addr, id.local_port)
        }
        Event::FileCreated(path, _) => format!("File created: {}", path.display()),
        Event::FileModified(path, _) => format!("File modified: {}", path.display()),
        Event::FileRemoved(path, _) => format!("File removed: {}", path.display()),
        Event::FileRenamed(old_path, new_path, _) => format!(
            "File renamed: {} -> {}",
            old_path.display(),
            new_path.display()
        ),
    }
}

pub fn format_time(time: SystemTime) -> String {
    let datetime: DateTime<Local> = time.into();
    datetime.format("%H:%M:%S").to_string()
}

pub fn category_color(category: EventCategory) -> Color32 {
    match category {
        EventCategory::Process => Color32::from_rgb(167, 139, 250),
        EventCategory::Network => Color32::from_rgb(34, 211, 238),
        EventCategory::File => Color32::from_rgb(74, 222, 128),
    }
}

pub fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Low => "LOW",
        Severity::Medium => "MEDIUM",
        Severity::High => "HIGH",
        Severity::Critical => "CRITICAL",
    }
}

pub fn severity_color(severity: Severity) -> Color32 {
    match severity {
        Severity::Low => Color32::from_rgb(148, 163, 184),
        Severity::Medium => Color32::from_rgb(250, 204, 21),
        Severity::High => Color32::from_rgb(251, 146, 60),
        Severity::Critical => Color32::from_rgb(239, 68, 68),
    }
}
