use std::ops::Deref;

use bytes::{Buf, BytesMut};

use crate::resp::decode::{RespDecode, CRLF_LEN};
use crate::resp::encode::{RespEncode, BUF_CAP};
use crate::resp::frame::RespFrame;
use crate::resp::{calc_total_length, parse_length};
use crate::RespError;

#[derive(Debug, Clone, PartialEq)]
pub struct RespSet(pub(crate) Vec<RespFrame>);

impl RespSet {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        let s = s.into();

        RespSet(s)
    }
}

impl RespEncode for RespSet {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("~{}\r\n", self.len()).into_bytes());
        for value in &self.0 {
            buf.extend_from_slice(&value.encode());
        }
        buf
    }
}

impl RespDecode for RespSet {
    const PREFIX: &'static str = "~";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;

        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);
        let mut frames = Vec::new();
        for _ in 0..len {
            let value = RespFrame::decode(buf)?;
            frames.push(value);
        }

        Ok(RespSet::new(frames))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl Deref for RespSet {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[cfg(test)]
mod tests {
    use crate::array::RespArray;
    use crate::bulk_string::BulkString;
    use crate::frame::RespFrame;

    use super::*;

    #[test]
    fn test_set_encode() {
        let frame: RespFrame = RespSet::new([
            RespArray::new([1234.into(), true.into()]).into(),
            BulkString::new("world".to_string()).into(),
        ])
        .into();
        // assert_eq!(String::from_utf8_lossy(&frame.encode()), "~2\r\n*2\r\n:1234\r\n#t\r\n$5\r\nworld\r\n");
        assert_eq!(
            frame.encode(),
            b"~2\r\n*2\r\n:1234\r\n#t\r\n$5\r\nworld\r\n"
        );
    }
    #[test]
    fn test_set_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"~2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespSet::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespSet::new(vec![
                BulkString::new(b"set".to_vec()).into(),
                BulkString::new(b"hello".to_vec()).into()
            ])
        );

        Ok(())
    }
}
