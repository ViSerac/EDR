use crate::collectors::file::file_change_kind::FileChangeKind;

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RawFileEvent {
    pub kind: FileChangeKind,
    pub file_name: String,
    pub watched_path: PathBuf,
}