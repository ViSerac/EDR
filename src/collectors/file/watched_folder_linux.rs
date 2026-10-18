use crate::collectors::file::raw_file_data::RawFileEvent;
use crate::collectors::file::watched_path::WatchedPath;
use std::sync::mpsc;

// Stub until the inotify collector exists. The sender is dropped immediately, so the
// receiver is disconnected and FileMonitor::poll_events just yields no events.
pub fn start_file_watchers(_watched_paths: Vec<WatchedPath>) -> mpsc::Receiver<RawFileEvent> {
    let (_sender, receiver) = mpsc::channel();
    receiver
}
