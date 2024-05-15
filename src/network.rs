use anyhow::Result;
use bytes::BytesMut;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::info;

use crate::backend::Backend;
use crate::cmd::{Command, CommandExecutor};
use crate::{RespDecode, RespEncode, RespError, RespFrame, SimpleString};

#[derive(Debug)]
pub struct RespFrameCodec {}

#[derive(Debug)]
pub struct RedisRequest {
    frame: RespFrame,
    backend: Backend,
}

#[derive(Debug)]
pub struct RedisResponse {
    frame: RespFrame,
}

pub async fn stream_handler(tcp_stream: TcpStream, backend: Backend) -> Result<()> {
    // how to get a frame from the tcp stream?

    let mut framed = Framed::new(tcp_stream, RespFrameCodec {});
    loop {
        match framed.next().await {
            Some(Ok(frame)) => {
                info!("received frame: {:?}", frame);
                let request = RedisRequest {
                    frame,
                    backend: backend.clone(),
                };
                // call request_handler with the frame
                let response = request_handler(request).await?;
                info!("sending response: {:?}", response);
                // send the response back to the tcp stream

                framed.send(response.frame).await?;
                // send the response back to the tcp stream
            }
            Some(Err(e)) => framed.send(SimpleString::new(e.to_string()).into()).await?,
            None => return Ok(()),
        }
    }
}

async fn request_handler(requset: RedisRequest) -> Result<RedisResponse> {
    let (frame, backend) = (requset.frame, requset.backend);
    match Command::try_from(frame) {
        Ok(cmd) => {
            info!("Executing command: {:?}", cmd);
            match cmd.execute(&backend) {
                Ok(frame) => Ok(RedisResponse { frame }),
                Err(e) => Ok(RedisResponse {
                    frame: SimpleString::new(e.to_string()).into(),
                }),
            }
        }
        Err(e) => Ok(RedisResponse {
            frame: SimpleString::new(e.to_string()).into(),
        }),
    }
}

impl Encoder<RespFrame> for RespFrameCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: RespFrame, dst: &mut BytesMut) -> Result<()> {
        let encoded = item.encode();
        dst.extend_from_slice(&encoded);
        Ok(())
    }
}
impl Decoder for RespFrameCodec {
    type Item = RespFrame;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        match RespFrame::decode(src) {
            Ok(frame) => Ok(Some(frame)),
            Err(RespError::NotComplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
