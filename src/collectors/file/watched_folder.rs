use crate::collectors::file::watched_path::WatchedPath;
use crate::collectors::file::raw_file_data::RawFileEvent;
use crate::collectors::file::file_change_kind::FileChangeKind;
use crate::models::tcp_connection::TcpState::Unknown;
use std::sync::mpsc;
use std::thread;
use std::ptr::addr_of;

use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_LIST_DIRECTORY, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SHARE_DELETE,
    OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, ReadDirectoryChangesW, FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_NOTIFY_INFORMATION
};
use windows::core::PCWSTR;

fn watch_folder(watched_path: WatchedPath, sender: mpsc::Sender<RawFileEvent>) {
    let path_str = watched_path.path.to_string_lossy();
    let wide: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();

    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            FILE_LIST_DIRECTORY.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )
    };

    let handle = match handle {
        Ok(h) => h,
        Err(error) => {
            eprintln!("Error opening directory handle for {:?}: {:?}", watched_path.path, error);
            return;
        }
    };

    let mut buffer = [0u8; 4096];

    loop {
        let mut bytes_returned: u32 = 0;

        let result = unsafe {
            ReadDirectoryChangesW(
                handle,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                watched_path.recursive,
                FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_LAST_WRITE,
                Some(&mut bytes_returned),
                None,
                None,
            )
        };

        match result {
            Ok(()) => {
                let mut offset: usize = 0;

                while (bytes_returned as usize) > 0 {
                    let entry_ptr = unsafe {
                        buffer.as_ptr().add(offset) as *const FILE_NOTIFY_INFORMATION
                    };

                    let next_entry_offset = unsafe { (*entry_ptr).NextEntryOffset };
                    let action = unsafe { (*entry_ptr).Action };
                    let file_name_length = unsafe { (*entry_ptr).FileNameLength };

                    let name_ptr = unsafe { std::ptr::addr_of!((*entry_ptr).FileName) as *const u16 };
                    let name_len_u16 = (file_name_length / 2) as usize;
                    let name_slice = unsafe { std::slice::from_raw_parts(name_ptr, name_len_u16) };
                    let file_name = String::from_utf16_lossy(name_slice);

                    let kind = match action.0 {
                        1 => Some(FileChangeKind::Added),
                        2 => Some(FileChangeKind::Removed),
                        3 => Some(FileChangeKind::Modified),
                        4 => Some(FileChangeKind::RenamedOldName),
                        5 => Some(FileChangeKind::RenamedNewName),
                        other => {
                            eprintln!("Warning: unknown file action code: {}", other);
                            None
                        }
                    };

                    if let Some(kind) = kind {
                        let raw_event = RawFileEvent {
                            kind,
                            file_name,
                            watched_path: watched_path.path.clone(),
                        };
                        let _ = sender.send(raw_event);
                    }

                    if next_entry_offset == 0 {
                        break;
                    }

                    offset += next_entry_offset as usize;
                }
            }
            Err(error) => {
                eprintln!("Error reading directory changes for {:?}: {:?}", watched_path.path, error);
                continue;
            }
        }
    }
}

pub fn start_file_watchers(watched_paths: Vec<WatchedPath>) -> mpsc::Receiver<RawFileEvent> {
    let (sender, receiver) = mpsc::channel();

    for watched_path in watched_paths {
        let thread_sender = sender.clone();
        thread::spawn(move || {
            watch_folder(watched_path, thread_sender);
        });
    }

    receiver
}