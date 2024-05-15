use std::ops::Deref;

use bytes::BytesMut;

use crate::resp::decode::{RespDecode, CRLF_LEN};
use crate::resp::encode::RespEncode;
use crate::resp::extract_simple_frame_data;
use crate::RespError;

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleString(pub(crate) String);

// - simple string: "+OK\r\n"
impl RespEncode for SimpleString {
    fn encode(&self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}

impl RespDecode for SimpleString {
    const PREFIX: &'static str = "+";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;

        // split the buffer

        let data = buf.split_to(end + crate::resp::decode::CRLF_LEN);
        let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);

        Ok(SimpleString::new(s.to_string()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl Deref for SimpleString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for SimpleString {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string())
    }
}

impl AsRef<str> for SimpleString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;

    use crate::frame::RespFrame;

    use super::*;

    #[test]
    fn test_simple_string_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::from("+OK\r\n");
        let frame = SimpleString::decode(&mut buf)?;
        assert_eq!(frame, SimpleString::new("OK".to_string()));

        let mut buf = BytesMut::from("+hello\r");

        let ret = SimpleString::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.put_u8(b'\n');
        assert_eq!(
            SimpleString::decode(&mut buf)?,
            SimpleString::new("hello".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_simple_string_encode() {
        let frame: RespFrame = SimpleString::new("OK".to_string()).into();
        assert_eq!(frame.encode(), b"+OK\r\n");
    }
}
