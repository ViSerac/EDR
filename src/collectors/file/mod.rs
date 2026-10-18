pub mod watched_path;
pub mod raw_file_data;
pub mod file_change_kind;
pub mod file_monitor;

#[cfg(windows)]
#[path = "watched_folder_windows.rs"]
pub mod watched_folder;

#[cfg(target_os = "linux")]
#[path = "watched_folder_linux.rs"]
pub mod watched_folder;
