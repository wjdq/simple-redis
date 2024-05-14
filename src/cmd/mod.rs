use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use thiserror::Error;

use crate::backend::Backend;
use crate::{RespArray, RespError, RespFrame, SimpleString};

mod hmap;
mod map;

lazy_static! {
    static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("invalid command : {0}")]
    InvalidCommand(String),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("resp error : {0}")]
    RespError(#[from] RespError),
    #[error("UTF8 error : {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

#[enum_dispatch]
pub trait CommandExecutor {
    fn execute(self, backend: &Backend) -> RespFrame;
}

// #[enum_dispatch(CommandExecutor)]
pub enum Command {
    Get(Get),
    Set(Set),
    HGet(HGet),
    HSet(HSet),
    HGetAll(HGetAll),
}

#[derive(Debug)]
pub struct Get {
    key: String,
}

#[derive(Debug)]
pub struct Set {
    key: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct HGet {
    #[allow(dead_code)]
    key: String,
    #[allow(dead_code)]
    field: String,
}

#[derive(Debug)]
pub struct HSet {
    #[allow(dead_code)]
    key: String,
    #[allow(dead_code)]
    field: String,
    #[allow(dead_code)]
    value: RespFrame,
}

#[derive(Debug)]
pub struct HGetAll {
    #[allow(dead_code)]
    key: String,
}

fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    // test if array has 2 elements
    if value.len() != n_args + names.len() {
        return Err(CommandError::InvalidArgument(format!(
            "{} command must have exactly {} argument",
            names.join(" "),
            n_args
        )));
    }

    for (i, name) in names.iter().enumerate() {
        match value[i] {
            RespFrame::BulkString(ref cmd) => {
                if cmd.as_ref().to_ascii_lowercase() != name.as_bytes() {
                    return Err(CommandError::InvalidCommand(format!(
                        "Invalid command: expected {} but got {}",
                        name,
                        String::from_utf8_lossy(cmd.as_ref())
                    )));
                }
            }
            _ => {
                return Err(CommandError::InvalidCommand(
                    "Command must have a BulkString as the first argument".to_string(),
                ))
            }
        }
    }

    Ok(())
}

fn extract_args(value: RespArray, start: usize) -> Result<Vec<RespFrame>, CommandError> {
    Ok(value.0.into_iter().skip(start).collect())
}
