#[derive(Debug, Clone)]
pub enum FileChangeKind {
    Added,
    Removed,
    Modified,
    RenamedOldName,
    RenamedNewName,
}