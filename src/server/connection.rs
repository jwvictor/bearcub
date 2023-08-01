use std::io::Cursor;
use std::fmt;

use bytes::{BytesMut, Buf};
use tokio::{net::TcpStream, io::{AsyncReadExt, AsyncWriteExt}};
use anyhow::*;

use crate::protocol::{types::*, wire::{Frame, check_frame, try_parse_frame}};

pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            // Allocate the buffer with enough capacity to hold 4 frames.
            buffer: BytesMut::with_capacity(BUF_CAP * 4),
        }
    }

    pub fn parse_frame(&mut self) -> Result<Option<Frame>> {
        let buf_len = self.buffer.len();
        println!("parse_frame has {} bytes", buf_len);
        let mut buf = Cursor::new(&self.buffer[..]);
        let fr_sz = check_frame(&mut buf, buf_len).with_context(|| "check_frame returned false")?;
        buf.set_position(0);
        let fr = try_parse_frame(&mut buf, buf_len).with_context(|| "parse error")?;
        self.buffer.advance(fr_sz);
        Ok(Some(fr))
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame().ok() {
                return Ok(frame)
            }

            println!("Insufficient data in buffer: {}", String::from_utf8_lossy(&self.buffer.to_vec()[..]));
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(anyhow!("read error"));
                }
            }
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> Result<usize> {
        let bs = frame.to_bytes();
        // self.stream.writable().await?;
        // self.stream.try_write(&bs[..]).with_context(|| "write error")
        let n = self.stream.write(&bs[..]).await;
        if n.is_ok() {
            Ok(n.unwrap())
        } else {
            Err(anyhow!("stream write err"))
        }
    }
}

