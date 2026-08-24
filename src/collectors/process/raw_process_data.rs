const MAX_PATH: usize = 260;

#[derive(Debug)]
pub struct RawProcessData {
    pub pid: u32,
    pub parent_pid: u32,
    pub creation_time: Option<u64>,
    pub exe_file: [u8; MAX_PATH],
    pub path: Option<String>,
    pub command_line: Option<[u8; MAX_PATH]>,
    pub user: Option<[u8; MAX_PATH]>,
}
