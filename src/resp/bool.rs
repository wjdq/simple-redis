use bytes::BytesMut;

use crate::resp::decode::RespDecode;
use crate::resp::encode::RespEncode;
use crate::resp::extract_fixed_data;
use crate::RespError;

impl RespEncode for bool {
    fn encode(&self) -> Vec<u8> {
        format!("#{}\r\n", if *self { "t" } else { "f" }).into_bytes()
    }
}
impl RespDecode for bool {
    const PREFIX: &'static str = "#";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        match extract_fixed_data(buf, "#t\r\n", "Bool") {
            Ok(_) => Ok(true),
            Err(_) => match extract_fixed_data(buf, "#f\r\n", "Bool") {
                Ok(_) => Ok(false),
                Err(e) => Err(e),
            },
        }
    }

    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(4)
    }
}
#[cfg(test)]
mod tests {
    use crate::frame::RespFrame;

    use super::*;

    #[test]
    fn test_bool_encode() {
        let frame: RespFrame = true.into();
        assert_eq!(frame.encode(), b"#t\r\n");
        let frame: RespFrame = false.into();
        assert_eq!(frame.encode(), b"#f\r\n");
    }
    #[test]
    fn test_bool_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::from("#t\r\n");
        let frame = bool::decode(&mut buf)?;
        assert!(frame);

        let mut buf = BytesMut::from("#f\r\n");
        let frame = bool::decode(&mut buf)?;
        assert!(!frame);
        Ok(())
    }
}
