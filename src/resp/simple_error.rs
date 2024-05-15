use std::ops::Deref;

use bytes::BytesMut;

use crate::resp::decode::{RespDecode, CRLF_LEN};
use crate::resp::encode::RespEncode;
use crate::resp::extract_simple_frame_data;
use crate::RespError;

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleError(pub(crate) String);

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}

// impl trait
// - error: "-Error message\r\n"

impl RespEncode for SimpleError {
    fn encode(&self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

impl RespDecode for SimpleError {
    const PREFIX: &'static str = "-";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;

        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);
        Ok(SimpleError::new(s.to_string()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}
impl Deref for SimpleError {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<&str> for SimpleError {
    fn from(s: &str) -> Self {
        SimpleError(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::RespFrame;

    use super::*;

    #[test]
    fn test_simple_error_encode() {
        let frame: RespFrame = SimpleError::new("ERR".to_string()).into();
        assert_eq!(frame.encode(), b"-ERR\r\n");
    }

    #[test]
    fn test_simple_error_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::from("-ERR\r\n");
        let frame = SimpleError::decode(&mut buf)?;
        assert_eq!(frame, SimpleError::new("ERR".to_string()));

        Ok(())
    }
}
