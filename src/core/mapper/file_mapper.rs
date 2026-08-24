use crate::collectors::file::raw_file_data::RawFileEvent;
use crate::models::event::Event;
use crate::collectors::file::file_change_kind::FileChangeKind;

use std::time::SystemTime;

pub fn map_raw_file_events(raw_events: Vec<RawFileEvent>) -> Vec<Event> {
    let mut events = Vec::new();
    let mut i = 0;

    while i < raw_events.len() {
        let raw = &raw_events[i];

        match raw.kind {
            FileChangeKind::RenamedOldName => {
                match raw_events.get(i + 1) {
                    Some(next_raw) => {
                        let old_path = raw.watched_path.join(&raw.file_name);
                        let new_path = next_raw.watched_path.join(&next_raw.file_name);
                        events.push(Event::FileRenamed(old_path, new_path, SystemTime::now()));
                        i += 2;
                    }
                    None => {
                        eprintln!("Warning: RenamedOldName without a following RenamedNewName");
                        i += 1;
                    }
                }
            }
            FileChangeKind::Added => {
                let full_path = raw.watched_path.join(&raw.file_name);
                events.push(Event::FileCreated(full_path, SystemTime::now()));
                i += 1;
            }
            FileChangeKind::Modified => {
                let full_path = raw.watched_path.join(&raw.file_name);
                events.push(Event::FileModified(full_path, SystemTime::now()));
                i += 1;
            }
            FileChangeKind::Removed => {
                let full_path = raw.watched_path.join(&raw.file_name);
                events.push(Event::FileRemoved(full_path, SystemTime::now()));
                i += 1;
            }
            FileChangeKind::RenamedNewName => {
                eprintln!("Warning: orphan RenamedNewName without a preceding RenamedOldName: {:?}", raw.file_name);
                i += 1;
            }
        }
    }

    events
}