use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

use bytes::{Buf, BytesMut};

use crate::resp::decode::{RespDecode, CRLF_LEN};
use crate::resp::encode::{RespEncode, BUF_CAP};
use crate::resp::frame::RespFrame;
use crate::resp::simple_string::SimpleString;
use crate::resp::{calc_total_length, parse_length};
use crate::RespError;

#[derive(Debug, Clone, PartialEq)]
pub struct RespMap(pub(crate) BTreeMap<String, RespFrame>);

impl RespMap {
    pub fn new() -> Self {
        RespMap(BTreeMap::new())
    }
}

impl Default for RespMap {
    fn default() -> Self {
        Self::new()
    }
}

// - map
impl RespEncode for RespMap {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("%{}\r\n", self.len()).into_bytes());
        for (key, value) in &self.0 {
            buf.extend_from_slice(&SimpleString::new(key).encode());
            buf.extend_from_slice(&value.encode());
        }

        buf
    }
}

impl RespDecode for RespMap {
    const PREFIX: &'static str = "%";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;
        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);
        let mut frames = RespMap::new();
        for _ in 0..len {
            let key = SimpleString::decode(buf)?;
            let value = RespFrame::decode(buf)?;
            frames.insert(key.0, value);
        }
        Ok(frames)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl Deref for RespMap {
    type Target = BTreeMap<String, RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
#[cfg(test)]
mod tests {
    use crate::bulk_string::BulkString;
    use crate::frame::RespFrame;

    use super::*;

    #[test]
    fn test_map_encode() {
        let mut map = RespMap::new();

        map.insert(
            "hello".to_string(),
            BulkString::new("world".to_string()).into(),
        );
        map.insert("foo".to_string(), (-123456.789).into());
        let frame: RespFrame = map.into();
        // assert_eq!(String::from_utf8_lossy(&frame.encode()), "%2\r\n+foo\r\n,-123456.789\r\n+hello\r\n$5\r\nworld\r\n");
        assert_eq!(
            frame.encode(),
            b"%2\r\n+foo\r\n,-123456.789\r\n+hello\r\n$5\r\nworld\r\n"
        );
    }
    #[test]
    fn test_map_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::from("%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n");
        let frame = RespMap::decode(&mut buf)?;

        let mut map = RespMap::new();
        map.insert(
            SimpleString::new("hello").0,
            BulkString::new(b"world".to_vec()).into(),
        );
        map.insert(
            SimpleString::new("foo").0,
            BulkString::new(b"bar".to_vec()).into(),
        );
        assert_eq!(frame, map);
        Ok(())
    }
}
