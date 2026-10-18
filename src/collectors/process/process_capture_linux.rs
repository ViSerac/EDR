use crate::collectors::process::raw_process_data::RawProcessData;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;

const ESRCH: i32 = 3;

#[derive(Debug)]
pub enum CaptureError {
    ProcUnreadable(io::Error),
}

struct StatFields {
    comm: Vec<u8>,
    parent_pid: u32,
    start_time: u64,
}

pub fn capture_raw_processes() -> Result<Vec<RawProcessData>, CaptureError> {
    let entries = match fs::read_dir("/proc") {
        Ok(entries) => entries,
        Err(error) => return Err(CaptureError::ProcUnreadable(error)),
    };

    let mut processes: Vec<RawProcessData> = Vec::new();

    for entry in entries {
        let Ok(entry) = entry else { continue };
        let Some(pid) = parse_pid(&entry.file_name()) else { continue };

        if let Some(raw) = read_process(pid) {
            processes.push(raw);
        }
    }

    Ok(processes)
}

fn parse_pid(name: &OsStr) -> Option<u32> {
    let name = name.to_str()?;
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    name.parse().ok()
}

fn read_process(pid: u32) -> Option<RawProcessData> {
    let proc_dir = Path::new("/proc").join(pid.to_string());

    let stat = match fs::read(proc_dir.join("stat")) {
        Ok(bytes) => bytes,
        Err(error) if process_vanished(&error) => return None,
        Err(error) => {
            eprintln!("Error reading /proc/{}/stat: {:?}", pid, error);
            return None;
        }
    };

    let Some(fields) = parse_stat(&stat) else {
        eprintln!("Malformed /proc/{}/stat, skipping: {:?}", pid, String::from_utf8_lossy(&stat));
        return None;
    };

    let path = fs::read_link(proc_dir.join("exe"))
        .ok()
        .and_then(|target| target.into_os_string().into_string().ok());

    Some(RawProcessData {
        pid,
        parent_pid: fields.parent_pid,
        creation_time: Some(fields.start_time),
        exe_file: fields.comm,
        path,
        command_line: None,
        user: None,
    })
}

fn process_vanished(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(ESRCH)
}

fn parse_stat(stat: &[u8]) -> Option<StatFields> {
    let open = stat.iter().position(|&b| b == b'(')?;
    let close = stat.iter().rposition(|&b| b == b')')?;
    if close < open {
        return None;
    }

    let comm = stat[open + 1..close].to_vec();

    let rest = std::str::from_utf8(&stat[close + 1..]).ok()?;
    let fields: Vec<&str> = rest.split_whitespace().collect();

    let parent_pid = fields.get(1)?.parse().ok()?;
    let start_time = fields.get(19)?.parse().ok()?;

    Some(StatFields { comm, parent_pid, start_time })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stat_line(comm: &str) -> Vec<u8> {
        format!(
            "1234 ({}) S 42 1234 1234 0 -1 4194560 100 0 0 0 1 2 0 0 20 0 1 0 987654 1000000 200 18446744073709551615",
            comm
        )
        .into_bytes()
    }

    #[test]
    fn parses_plain_stat() {
        let fields = parse_stat(&stat_line("bash")).unwrap();
        assert_eq!(fields.comm, b"bash");
        assert_eq!(fields.parent_pid, 42);
        assert_eq!(fields.start_time, 987654);
    }

    #[test]
    fn comm_with_spaces_and_parens_cannot_shift_fields() {
        let fields = parse_stat(&stat_line("x) S 1 1")).unwrap();
        assert_eq!(fields.comm, b"x) S 1 1");
        assert_eq!(fields.parent_pid, 42);
        assert_eq!(fields.start_time, 987654);
    }

    #[test]
    fn rejects_stat_without_closing_paren() {
        assert!(parse_stat(b"1234 (bash S 42 1234").is_none());
    }

    #[test]
    fn rejects_truncated_stat() {
        assert!(parse_stat(b"1234 (bash) S 42").is_none());
    }

    #[test]
    fn parse_pid_accepts_only_digits() {
        assert_eq!(parse_pid(OsStr::new("1234")), Some(1234));
        assert_eq!(parse_pid(OsStr::new("self")), None);
        assert_eq!(parse_pid(OsStr::new("+12")), None);
        assert_eq!(parse_pid(OsStr::new("")), None);
    }

    #[test]
    fn captures_own_process() {
        let processes = capture_raw_processes().unwrap();
        let me = processes.iter().find(|p| p.pid == std::process::id()).unwrap();
        assert!(me.creation_time.is_some());
        assert!(me.path.is_some());
    }
}
