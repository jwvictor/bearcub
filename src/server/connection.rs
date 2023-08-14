use std::{io::Cursor, time::SystemTime};
use bytes::{BytesMut, Buf};
use tokio::{net::TcpStream, io::{AsyncReadExt, AsyncWriteExt}};
use anyhow::*;

use crate::protocol::{types::*, wire::{Frame, check_frame, try_parse_frame}};

const CXN_FIRST_BYTE_TIMEOUT_MS: u32 = 5000;
const CXN_RECENT_BYTE_TIMEOUT_MS: u32 = 5000;

pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut,
    last_recv_time: Option<SystemTime>,
    cxn_start_time: SystemTime,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            // Allocate the buffer with enough capacity to hold 4 frames.
            buffer: BytesMut::with_capacity(BUF_CAP * 4),
            last_recv_time: None,
            cxn_start_time: SystemTime::now(),
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
            let parse_res = self.parse_frame();
            if let Result::Ok(frame) = parse_res {
                return Ok(frame)
            } else {
                println!("Error parsing frame: {:?}", parse_res.unwrap_err());
            }

            println!("Insufficient data in buffer: {}", String::from_utf8_lossy(&self.buffer.to_vec()[..]));
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(anyhow!("read error"));
                }
            } else {
                let t = SystemTime::now();
                self.last_recv_time = Some(t);
            }
        }
    }

    pub fn is_timed_out(&self) -> bool {
        let t_now = SystemTime::now();
        match self.last_recv_time {
            Some(t) => {
                //
                let dur_diff = t_now.duration_since(t);
                dur_diff.unwrap().as_millis() > (CXN_RECENT_BYTE_TIMEOUT_MS as u128)
            },
            None => {
                // if it's none, 0 bytes have been received, so work off cxn start time
                let dur_diff = t_now.duration_since(self.cxn_start_time);
                dur_diff.unwrap().as_millis() > (CXN_FIRST_BYTE_TIMEOUT_MS as u128)
            },
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
            println!("error writing frame: {:?}", n.unwrap_err());
            Err(anyhow!("stream write err"))
        }
    }
}

