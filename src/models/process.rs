use crate::models::process_identity::ProcessIdentity;

#[derive(Debug, Clone)]
pub struct Process {
    pid: u32,
    parent_pid: u32,
    creation_time: Option<u64>,
    name: String,
    path: Option<String>,
    command_line: Option<String>,
    user: Option<String>,
}

impl Process {
    pub fn new(pid: u32,
        parent_pid: u32,
        creation_time: Option<u64>,
        name: String,
        path: Option<String>,
        command_line: Option<String>,
        user: Option<String>) -> Process {
            Process {
                pid,
                parent_pid,
                creation_time,
                name,
                path,
                command_line,
                user,
            }
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn parent_pid(&self) -> u32 {
        self.parent_pid
    }

    pub fn creation_time(&self) -> Option<u64> {
        self.creation_time
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub fn identity(&self) -> ProcessIdentity {
        ProcessIdentity {
            pid: self.pid,
            creation_time: self.creation_time,
        }
    }
    
}