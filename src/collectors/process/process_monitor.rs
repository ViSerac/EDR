use crate::models::{process_snapshot::ProcessSnapshot, process::Process, event::Event};
use crate::core::differ::snapshot_differ::{diff, build_process_events};


pub struct ProcessMonitor {
    last_snapshot: Option<ProcessSnapshot>,
}

impl ProcessMonitor {
    pub fn new() -> ProcessMonitor {
        Self {last_snapshot: None}
    }

    pub fn run_cycle(&mut self, current_processes: Vec<Process>) -> Vec<Event> {
        let new_snapshot = ProcessSnapshot::new(current_processes);

        let events = match self.last_snapshot.take() {
            Some(old_snapshot) => {
                let diff_result = diff(&old_snapshot, &new_snapshot);
                build_process_events(diff_result, &old_snapshot, &new_snapshot)
            }
            None => Vec::new()
        };

        self.last_snapshot = Some(new_snapshot);

        events
    }
}
