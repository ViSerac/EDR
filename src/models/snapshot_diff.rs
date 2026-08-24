#[derive(Debug, Clone)]
pub struct SnapshotDiff<I> {
    pub created: Vec<I>,
    pub terminated: Vec<I>,
}