use std::num::ParseIntError;

use bytes::{Buf, BytesMut};
use thiserror::Error;

use crate::decode::{RespDecode, CRLF_LEN};
use crate::frame::RespFrame;
use crate::simple_string::SimpleString;

pub mod array;
pub mod bool;
pub mod bulk_string;
pub mod decode;
pub mod double;
pub mod encode;
pub mod frame;
pub mod integer;
pub mod map;
pub mod null;
pub mod set;
pub mod simple_error;
pub mod simple_string;

#[derive(Debug, Error, PartialEq)]
pub enum RespError {
    #[error("Invalid frame : {0}")]
    InvalidFrame(String),
    #[error("Invalid frame type : {0}")]
    InvalidFrameType(String),
    #[error("Invalid frame length : {0}")]
    InvalidFrameLength(isize),
    #[error("Not complete")]
    NotComplete,

    #[error("parse int error : {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("parse float error : {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("UTF8 error : {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

pub(crate) fn extract_simple_frame_data(buf: &[u8], prefix: &str) -> Result<usize, RespError> {
    if buf.len() < 3 {
        return Err(RespError::NotComplete);
    }

    if !buf.starts_with(prefix.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect ({}), but got {:?}",
            prefix, buf
        )));
    }

    // search for "\r\n"
    let end = find_crlf(buf, 1).ok_or(RespError::NotComplete)?;
    Ok(end)
}

fn find_crlf(buf: &[u8], nth: usize) -> Option<usize> {
    let mut count = 0;
    for i in 0..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            count += 1;
            if count == nth {
                return Some(i);
            }
        }
    }
    None
}

pub(crate) fn extract_fixed_data(
    buf: &mut BytesMut,
    expect: &str,
    expect_type: &str,
) -> Result<(), RespError> {
    if buf.len() < expect.len() {
        return Err(RespError::NotComplete);
    }
    if !buf.starts_with(expect.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect {} but got {:?}",
            expect_type, buf
        )));
    }

    buf.advance(expect.len());
    Ok(())
}

pub(crate) fn parse_length(buf: &[u8], prefix: &str) -> Result<(usize, usize), RespError> {
    let end = extract_simple_frame_data(buf, prefix)?;
    let s = String::from_utf8_lossy(&buf[prefix.len()..end]);
    Ok((end, s.parse()?))
}

pub(crate) fn calc_total_length(
    buf: &[u8],
    end: usize,
    len: usize,
    prefix: &str,
) -> Result<usize, RespError> {
    let mut total = end + CRLF_LEN;
    let mut data = &buf[total..];
    match prefix {
        "*" | "~" => {
            for _ in 0..len {
                let len = RespFrame::expect_length(data)?;
                data = &data[len..];
                total += len;
            }
            Ok(total)
        }
        "%" => {
            for _ in 0..len {
                let len = SimpleString::expect_length(data)?;
                data = &data[len..];
                total += len;

                let len = RespFrame::expect_length(data)?;
                data = &data[len..];
                total += len;
            }
            Ok(total)
        }

        _ => Ok(len + CRLF_LEN),
    }
}
