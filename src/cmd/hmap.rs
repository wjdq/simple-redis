use crate::array::RespArray;
use crate::bulk_string::BulkString;
use crate::cmd::{
    extract_args, validate_command, CommandError, CommandExecutor, HGet, HGetAll, HSet, RESP_OK,
};
use crate::frame::RespFrame;
use crate::null::RespNull;

impl CommandExecutor for HGet {
    fn execute(self, backend: &crate::backend::Backend) -> Result<RespFrame, CommandError> {
        Ok(backend
            .hget(&self.key, &self.field)
            .unwrap_or(RespFrame::Null(RespNull)))
    }
}

impl CommandExecutor for HSet {
    fn execute(self, backend: &crate::backend::Backend) -> Result<RespFrame, CommandError> {
        backend.hset(self.key, self.field, self.value);
        Ok(RESP_OK.clone())
    }
}

impl CommandExecutor for HGetAll {
    fn execute(self, backend: &crate::backend::Backend) -> Result<RespFrame, CommandError> {
        let hmap = backend.hmap.get(&self.key);

        match hmap {
            Some(hmap) => {
                let mut data = Vec::with_capacity(hmap.len());
                for v in hmap.iter() {
                    let key = v.key().to_owned();
                    data.push((key, v.value().clone()))
                }
                if self.sort {
                    data.sort_by(|a, b| a.0.cmp(&b.0))
                }
                let ret = data
                    .into_iter()
                    .flat_map(|(k, v)| vec![BulkString::from(k).into(), v])
                    .collect::<Vec<RespFrame>>();

                Ok(RespArray::new(ret).into())
            }
            None => Ok(RespArray::new([]).into()),
        }
    }
}

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
                sort: false,
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

    use crate::decode::RespDecode;

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
    fn test_hset_hget_hgetall_command() -> Result<()> {
        let backend = crate::backend::Backend::new();
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*4\r\n$4\r\nhset\r\n$3\r\nmap\r\n$5\r\nhello\r\n$5\r\nworld\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let cmd: HSet = frame.try_into()?;
        let ret = cmd.execute(&backend);
        assert_eq!(ret?, RESP_OK.clone());
        buf.extend_from_slice(b"*4\r\n$4\r\nhset\r\n$3\r\nmap\r\n$6\r\nhello1\r\n$6\r\nworld1\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let cmd: HSet = frame.try_into()?;
        let ret = cmd.execute(&backend);
        assert_eq!(ret?, RESP_OK.clone());

        buf.extend_from_slice(b"*3\r\n$4\r\nhget\r\n$3\r\nmap\r\n$6\r\nhello1\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let cmd: HGet = frame.try_into()?;
        let ret = cmd.execute(&backend);
        assert_eq!(ret?, BulkString::new("world1").into());

        let cmd = HGetAll {
            key: "map".to_string(),
            sort: true,
        };
        let ret = cmd.execute(&backend);
        assert_eq!(
            ret?,
            RespArray::new([
                BulkString::new("hello").into(),
                BulkString::new("world").into(),
                BulkString::new("hello1").into(),
                BulkString::new("world1").into()
            ])
            .into()
        );

        Ok(())
    }
}
