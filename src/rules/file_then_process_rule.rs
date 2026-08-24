use std::time::SystemTime;
use std::path::PathBuf;
use std::collections::HashMap;

use crate::core::rules::rule_trait::Rule;
use crate::models::event::Event;
use crate::models::alert::Alert;
use crate::models::severity::Severity;

pub struct FileThenProcessRule {
    recent_files: HashMap<PathBuf, SystemTime>,
}

impl FileThenProcessRule {
    pub fn new() -> FileThenProcessRule {
        FileThenProcessRule {
            recent_files: HashMap::new(),
        }
    }
}

impl Rule for FileThenProcessRule {
    fn process(&mut self, event: &Event) -> Option<Alert> {
        match event {
            Event::FileCreated(path, timestamp) => {
                self.recent_files.insert(path.clone(), *timestamp);
                None
            }
            Event::ProcessCreated(process, timestamp) => {
                if let Some(process_path) = process.path() {
                    let path_buf = PathBuf::from(process_path);

                    if let Some(created_at) = self.recent_files.get(&path_buf) {
                        if let Ok(elapsed) = timestamp.duration_since(*created_at) {
                            if elapsed.as_secs() <= 60 {
                                return Some(Alert {
                                    rule_name: "FileThenProcess".to_string(),
                                    description: format!("Process executed shortly after file creation: {:?}", path_buf),
                                    events: vec![event.clone()],
                                    severity: Severity::High,
                                    detected_at: *timestamp,
                                });
                            }
                        }
                    }
                }

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
    use std::time::Duration;

    #[test]
    fn detects_process_running_from_recently_created_file() {
        let mut rule = FileThenProcessRule::new();

        let file_path = PathBuf::from(r"C:\Users\Test\AppData\Local\Temp\malware.exe");
        let file_time = SystemTime::now();

        let file_event = Event::FileCreated(file_path.clone(), file_time);
        let result1 = rule.process(&file_event);
        assert!(result1.is_none());

        let process = Process::new(
            500,
            1,
            None,
            "malware.exe".to_string(),
            Some(r"C:\Users\Test\AppData\Local\Temp\malware.exe".to_string()),
            None,
            None,
        );

        let process_time = file_time + Duration::from_secs(5);
        let process_event = Event::ProcessCreated(process, process_time);
        let result2 = rule.process(&process_event);
        assert!(result2.is_some());
    }
}