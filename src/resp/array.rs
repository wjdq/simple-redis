use std::ops::Deref;

use bytes::{Buf, BytesMut};

use crate::resp::decode::{RespDecode, CRLF_LEN};
use crate::resp::encode::{RespEncode, BUF_CAP};
use crate::resp::frame::RespFrame;
use crate::resp::{calc_total_length, extract_fixed_data, parse_length};
use crate::RespError;

#[derive(Debug, Clone, PartialEq)]
pub struct RespArray(pub(crate) Vec<RespFrame>);

#[derive(Debug, Clone, PartialEq)]
pub struct RespNullArray;

impl RespArray {
    pub fn new(v: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(v.into())
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// - array: "*<length>\r\n"
impl RespEncode for RespArray {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("*{}\r\n", self.len()).into_bytes());
        for frame in &self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}
impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;
        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);

        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            frames.push(RespFrame::decode(buf)?);
        }
        Ok(RespArray::new(frames))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

// - null array: "*0\r\n"
impl RespEncode for RespNullArray {
    fn encode(&self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}

impl RespDecode for RespNullArray {
    const PREFIX: &'static str = "*";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "*-1\r\n", "NullArray")?;
        Ok(RespNullArray)
    }

    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(4)
    }
}

#[cfg(test)]
mod tests {
    use crate::bulk_string::BulkString;
    use crate::frame::RespFrame;

    use super::*;

    #[test]
    fn test_array_encode() {
        let frame: RespFrame = RespArray::new(vec![
            BulkString::new("set".to_string()).into(),
            BulkString::new("hello").into(),
            BulkString::new("world").into(),
        ])
        .into();
        // assert_eq!(String::from_utf8_lossy(&frame.encode()), "*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");
        assert_eq!(
            frame.encode(),
            b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
        );
    }
    #[test]
    fn test_array_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");
        let ret = RespArray::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        Ok(())
    }

    #[test]
    fn test_null_array_encode() {
        let frame: RespFrame = RespNullArray.into();
        assert_eq!(frame.encode(), b"*-1\r\n");
    }
}
