pub mod raw_process_data;
pub mod process_monitor;

#[cfg(windows)]
#[path = "process_capture_windows.rs"]
pub mod process_capture;

#[cfg(target_os = "linux")]
#[path = "process_capture_linux.rs"]
pub mod process_capture;