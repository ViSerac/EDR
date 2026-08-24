use crate::collectors::file::raw_file_data::RawFileEvent;
use crate::models::event::Event;
use crate::core::mapper::file_mapper::map_raw_file_events;
use std::sync::mpsc;

pub struct FileMonitor {
    receiver: mpsc::Receiver<RawFileEvent>,
}

impl FileMonitor {
    pub fn new(receiver: mpsc::Receiver<RawFileEvent>) -> FileMonitor {
        FileMonitor { receiver }
    }

    pub fn poll_events(&mut self) -> Vec<Event> {
        let mut raw_events = Vec::new();

        while let Ok(raw_event) = self.receiver.try_recv() {
            raw_events.push(raw_event);
        }

        map_raw_file_events(raw_events)
    }
}