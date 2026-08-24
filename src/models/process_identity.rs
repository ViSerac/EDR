#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct ProcessIdentity {
    pub pid: u32,
    pub creation_time: Option<u64>,
}