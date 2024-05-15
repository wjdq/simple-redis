use bytes::BytesMut;

use crate::decode::RespDecode;
use crate::encode::RespEncode;
use crate::resp::extract_fixed_data;
use crate::RespError;

#[derive(Debug, Clone, PartialEq)]
pub struct RespNull;

impl RespDecode for RespNull {
    const PREFIX: &'static str = "_";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "_\r\n", "Null")?;

        Ok(RespNull)
    }

    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(3)
    }
}

// - null : "_\r\n"

impl RespEncode for RespNull {
    fn encode(&self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}
#[cfg(test)]
mod tests {
    use crate::frame::RespFrame;

    use super::*;

    #[test]
    fn test_null_encode() {
        let frame: RespFrame = RespNull.into();
        assert_eq!(frame.encode(), b"_\r\n");
    }
}
