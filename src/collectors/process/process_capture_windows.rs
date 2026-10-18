use crate::collectors::process::raw_process_data::RawProcessData;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::System::Threading::GetProcessTimes;
use windows::Win32::Foundation::FILETIME;

use windows::Win32::System::Threading::{QueryFullProcessImageNameW, PROCESS_NAME_FORMAT};
use windows::core::PWSTR;

#[derive(Debug)]
pub enum CaptureError {
    SnapshotFailed,
    FirstProcessFailed,
}

pub fn capture_raw_processes() -> Result<Vec<RawProcessData>, CaptureError> {
    let snapshot_handle = unsafe {
    match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
        Ok(handle) => handle,
        Err(_) => return Err(CaptureError::SnapshotFailed),
        }
    };

    let mut process_entry = PROCESSENTRY32::default();
    process_entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

    unsafe {
        match Process32First(snapshot_handle, &mut process_entry) {
            Ok(()) => {},
            Err(_) => return Err(CaptureError::FirstProcessFailed),
        }
    }

    let mut processes: Vec<RawProcessData> = Vec::new();
    processes.push(processentry_to_raw(&process_entry));

    loop {
    unsafe {
        match Process32Next(snapshot_handle, &mut process_entry) {
            Ok(()) => {
                processes.push(processentry_to_raw(&process_entry));
            }
            Err(_) => break,
            }
        }
    }

    let _ = unsafe {CloseHandle(snapshot_handle)};
    Ok(processes)
}

fn processentry_to_raw(entry: &PROCESSENTRY32) -> RawProcessData {
    let exe_file: Vec<u8> = entry
        .szExeFile
        .iter()
        .map(|&c| c as u8)
        .take_while(|&b| b != 0)
        .collect();

    RawProcessData {
        pid: entry.th32ProcessID,
        parent_pid: entry.th32ParentProcessID,
        creation_time: get_process_creation_time(entry.th32ProcessID),
        exe_file,
        path: get_process_path(entry.th32ProcessID),
        command_line: None,
        user: None,
    }
}

fn get_process_creation_time(pid: u32) -> Option<u64> {
    let handle = unsafe {
        match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return None,
        }
    };

    let mut creation_time = FILETIME::default();
    let mut exit_time = FILETIME::default();
    let mut kernel_time = FILETIME::default();
    let mut user_time = FILETIME::default();

    match unsafe {GetProcessTimes(handle, &mut creation_time, &mut exit_time, &mut kernel_time, &mut user_time)} {
        Ok(()) => {
            let high = creation_time.dwHighDateTime;
            let low = creation_time.dwLowDateTime;
            let combined_times = (high as u64) << 32 | (low as u64);
            let _ = unsafe {CloseHandle(handle)};
            return Some(combined_times)
        },
        Err(_) => {
            let _ = unsafe {CloseHandle(handle)};
            return None;
        },
    }
}

fn get_process_path(pid: u32) -> Option<String> {
    let handle = unsafe {
        match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return None,
        }
    };

    let mut buffer: [u16; 260] = [0; 260];
    let mut size: u32 = buffer.len() as u32;

    let result = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )
    };

    match result {
        Ok(()) => {
            let trimmed = &buffer[..size as usize];
            let path_result = match String::from_utf16(trimmed) {
                Ok(value) => Some(value),
                Err(_) => return None
            };

            let _ = unsafe {CloseHandle(handle)};
            return path_result
        },
        Err(_) => {
            let _ = unsafe {CloseHandle(handle)};
            return None
        }
    }
}