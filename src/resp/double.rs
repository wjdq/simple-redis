use bytes::BytesMut;

use crate::resp::decode::{RespDecode, CRLF_LEN};
use crate::resp::encode::RespEncode;
use crate::resp::extract_simple_frame_data;
use crate::RespError;

impl RespEncode for f64 {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);
        let ret = if self.abs() > 1e+8 || self.abs() < 1e-8 {
            format!(",{:+e}\r\n", self)
        } else {
            let sign = if self < &0.0 { "" } else { "+" };
            format!(",{}{}\r\n", sign, self)
        };

        buf.extend_from_slice(&ret.into_bytes());
        buf
    }
}

impl RespDecode for f64 {
    const PREFIX: &'static str = ",";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;

        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);
        Ok(s.parse()?)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}
#[cfg(test)]
mod tests {
    use crate::frame::RespFrame;

    use super::*;

    #[test]
    fn test_double_encode() {
        let frame: RespFrame = 123.456.into();
        assert_eq!(frame.encode(), b",+123.456\r\n");
        let frame: RespFrame = (-123.456).into();
        assert_eq!(frame.encode(), b",-123.456\r\n");
        let frame: RespFrame = 1.23456e+8.into();
        // assert_eq!(String::from_utf8_lossy(&frame.encode()), ",+1.23456e8\r\n");
        assert_eq!(frame.encode(), b",+1.23456e8\r\n");
        let frame: RespFrame = (-1.23456e-9).into();
        // assert_eq!(String::from_utf8_lossy(&frame.encode()), ",-1.23456e-9\r\n");
        assert_eq!(frame.encode(), b",-1.23456e-9\r\n");
    }

    #[test]
    fn test_double_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::from(",123.45\r\n");
        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, 123.45);

        let mut buf = BytesMut::from(",+1.23456e-9\r\n");

        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, 1.23456e-9);

        Ok(())
    }
}
