use bytes::BytesMut;

use crate::resp::decode::{RespDecode, CRLF_LEN};
use crate::resp::encode::RespEncode;
use crate::resp::extract_simple_frame_data;
use crate::RespError;

// - integer: ":[<+|->]<value>\r\n"
impl RespEncode for i64 {
    fn encode(&self) -> Vec<u8> {
        format!(":{}\r\n", self).into_bytes()
    }
}

impl RespDecode for i64 {
    const PREFIX: &'static str = ":";

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
    fn test_integer_encode() {
        let frame: RespFrame = 123.into();
        // assert_eq!(String::from_utf8_lossy(&frame.encode()), ":123\r\n");
        assert_eq!(frame.encode(), b":123\r\n");

        let frame: RespFrame = (-123).into();

        // assert_eq!(String::from_utf8_lossy(&frame.encode()), ":-123\r\n");
        assert_eq!(frame.encode(), b":-123\r\n");
    }
    #[test]
    fn test_integer_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::from(":123\r\n");
        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, 123);

        let mut buf = BytesMut::from(":massless\r\n");

        let ret = i64::decode(&mut buf);
        println!("{:?}", ret);
        assert!(ret.is_err());
        Ok(())
    }
}
