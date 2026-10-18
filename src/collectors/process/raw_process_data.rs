#[derive(Debug)]
pub struct RawProcessData {
    pub pid: u32,
    pub parent_pid: u32,
    pub creation_time: Option<u64>,
    pub exe_file: Vec<u8>,
    pub path: Option<String>,
    pub command_line: Option<Vec<u8>>,
    pub user: Option<Vec<u8>>,
}
