use std::string::FromUtf8Error;
use crate::collectors::process::raw_process_data::RawProcessData;
use crate::models::process::Process;

 #[derive(Debug)]
pub enum MapperError {
    InvalidExeFile(FromUtf8Error),
    InvalidCommandLine(FromUtf8Error),
    InvalidUser(FromUtf8Error),
}

impl TryFrom<RawProcessData> for Process {
    type Error = MapperError;

    fn try_from(raw: RawProcessData) -> Result<Self, Self::Error> {
        let name_str = match String::from_utf8(raw.exe_file) {
            Ok(value) => value,
            Err(error) => return Err(MapperError::InvalidExeFile(error)),
        };

        let command_line_str = match raw.command_line {
            None => None,
            Some(bytes) => match String::from_utf8(bytes) {
                Ok(value) => Some(value),
                Err(error) => return Err(MapperError::InvalidCommandLine(error)),
            },
        };

        let user_str = match raw.user {
            None => None,
            Some(bytes) => match String::from_utf8(bytes) {
                Ok(value) => Some(value),
                Err(error) => return Err(MapperError::InvalidUser(error)),
            },
        };

        Ok(Process::new(
            raw.pid,
            raw.parent_pid,
            raw.creation_time,
            name_str,
            raw.path,
            command_line_str,
            user_str,
        ))
    }
}
