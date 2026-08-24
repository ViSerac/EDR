use crate::core::rules::rule_trait::Rule;
use crate::models::event::Event;
use crate::models::alert::Alert;
use crate::models::severity::Severity;

use std::collections::HashMap;

const SENSITIVE_SHELLS: [&str; 2] = ["powershell.exe", "cmd.exe"];
const EXPECTED_PARENTS: [&str; 5] = ["explorer.exe", "cmd.exe", "powershell.exe", "WindowsTerminal.exe", "Code.exe"];

pub struct SuspiciousParentRule {
    known_processes: HashMap<u32, String>,
}

impl SuspiciousParentRule {
    pub fn new() -> SuspiciousParentRule {
        SuspiciousParentRule {
            known_processes: HashMap::new(),
        }
    }
}

impl Rule for SuspiciousParentRule {
    fn process(&mut self, event: &Event) -> Option<Alert> {
        match event {
            Event::ProcessCreated(process, timestamp) => {
                self.known_processes.insert(process.pid(), process.name().to_string());

                let parent_name = self.known_processes.get(&process.parent_pid());

                if SENSITIVE_SHELLS.contains(&process.name()) {
                    if let Some(parent) = parent_name {
                        if !EXPECTED_PARENTS.contains(&parent.as_str()) {
                            return Some(Alert {
                                rule_name: "SuspiciousParent".to_string(),
                                description: format!("{} spawned by unexpected parent: {}", process.name(), parent),
                                events: vec![event.clone()],
                                severity: Severity::High,
                                detected_at: *timestamp,
                            });
                        }
                    }
                }

                None
            }
            Event::ProcessTerminated(process, _) => {
                self.known_processes.remove(&process.pid());
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::process::Process;
    use std::time::SystemTime;

    #[test]
    fn detects_cmd_spawned_by_unusual_parent() {
        let parent = Process::new(
            100,
            1,
            None,
            "weird_app.exe".to_string(),
            None,
            None,
            None,
        );

        let child = Process::new(
            200,
            100,
            None,
            "cmd.exe".to_string(),
            None,
            None,
            None,
        );

        let mut rule = SuspiciousParentRule::new();

        let parent_event = Event::ProcessCreated(parent, SystemTime::now());
        let result1 = rule.process(&parent_event);
        assert!(result1.is_none());

        let child_event = Event::ProcessCreated(child, SystemTime::now());
        let result2 = rule.process(&child_event);
        assert!(result2.is_some());
    }
}