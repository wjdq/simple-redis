use crate::cmd::{extract_args, validate_command, CommandError, HGet, HGetAll, HSet};
use crate::{RespArray, RespFrame};

impl TryFrom<RespArray> for HGet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hget"], 2)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field))) => Ok(HGet {
                key: String::from_utf8(key.0)?,
                field: String::from_utf8(field.0)?,
            }),

            _ => Err(CommandError::InvalidArgument(
                "Invalid key or value".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HSet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hset"], 3)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field)), Some(value)) => {
                Ok(HSet {
                    key: String::from_utf8(key.0)?,
                    field: String::from_utf8(field.0)?,
                    value,
                })
            }

            _ => Err(CommandError::InvalidArgument(
                "Invalid key or value".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HGetAll {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hgetall"], 1)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(HGetAll {
                key: String::from_utf8(key.0)?,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid key or filed or value".to_string(),
            )),
        }
    }
}
#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bytes::BytesMut;

    use crate::RespDecode;

    use super::*;

    #[test]
    fn test_hget_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$4\r\nhget\r\n$4\r\nkey1\r\n$6\r\nfield1\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let result: HGet = frame.try_into()?;
        assert_eq!(result.key, "key1");
        assert_eq!(result.field, "field1");
        Ok(())
    }

    #[test]
    fn test_hset_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(
            b"*4\r\n$4\r\nhset\r\n$4\r\nkey1\r\n$6\r\nfield1\r\n$6\r\nvalue1\r\n",
        );
        let frame = RespArray::decode(&mut buf)?;
        let result: HSet = frame.try_into()?;
        assert_eq!(result.key, "key1");
        assert_eq!(result.field, "field1");
        assert_eq!(result.value, RespFrame::BulkString("value1".into()));

        Ok(())
    }

    #[test]
    fn test_hgetall_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$7\r\nhgetall\r\n$4\r\nkey1\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let result: HGetAll = frame.try_into()?;
        assert_eq!(result.key, "key1");

        Ok(())
    }
}
