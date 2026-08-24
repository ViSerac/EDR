use std::collections::HashMap;
use super::process::Process;
use super::process_identity::ProcessIdentity;
use crate::core::differ::snapshot_trait::Snapshot;

pub struct ProcessSnapshot {
    processes: HashMap<ProcessIdentity, Process>,
}

impl ProcessSnapshot {
    pub fn new(processes: Vec<Process>) -> ProcessSnapshot {
        let mut processes_map = HashMap::new();

        for process in processes {
            processes_map.insert(process.identity(), process);
        }

        Self { processes: processes_map }
    }
}

impl Snapshot for ProcessSnapshot {
    type Identity = ProcessIdentity;
    type Item = Process;

    fn identities(&self) -> Vec<ProcessIdentity> {
        self.processes.keys().cloned().collect()
    }

    fn get(&self, identity: &ProcessIdentity) -> Option<&Process> {
        self.processes.get(identity)
    }
}