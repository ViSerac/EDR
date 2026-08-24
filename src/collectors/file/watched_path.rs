use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WatchedPath {
    pub path: PathBuf,
    pub recursive: bool,
    pub ignored_extensions: Vec<String>,
}