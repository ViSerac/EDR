pub mod raw_tcp_data;
pub mod tcp_monitor;
pub mod raw_udp_data;
pub mod udp_monitor;

#[cfg(windows)]
#[path = "tcp_capture_windows.rs"]
pub mod tcp_capture;

#[cfg(target_os = "linux")]
#[path = "tcp_capture_linux.rs"]
pub mod tcp_capture;

#[cfg(windows)]
#[path = "udp_capture_windows.rs"]
pub mod udp_capture;

#[cfg(target_os = "linux")]
#[path = "udp_capture_linux.rs"]
pub mod udp_capture;
