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
        let end = raw.exe_file.iter().position(|&b| b == 0).unwrap_or(raw.exe_file.len());
        let trimmed = &raw.exe_file[..end];

        let name_str = match String::from_utf8(trimmed.to_vec()) {
            Ok(value) => value,
            Err(error) => return Err(MapperError::InvalidExeFile(error)),
        };

        let command_line_str = match raw.command_line {
            None => None,
            Some(bytes) => {
                let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                let trimmed = &bytes[..end];

                match String::from_utf8(trimmed.to_vec()) {
                    Ok(value) => Some(value),
                    Err(error) => return Err(MapperError::InvalidCommandLine(error)),
                }
            }
        };

        let user_str = match raw.user {
            None => None,
            Some(bytes) => {
                let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                let trimmed = &bytes[..end];
                match String::from_utf8(trimmed.to_vec()) {
                    Ok(value) => Some(value),
                    Err(error) => return Err(MapperError::InvalidUser(error))
                }
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