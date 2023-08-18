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

pub trait Frameable {
    fn to_frames(self) -> Vec<Frame>; 
    fn from_frames(_: Vec<Frame>) -> Result<Self>;
}

pub enum BearcubMessage {
    Request {
        msg: RequestMessage,
    },
    Response {
        msg: ResponseMessage,
    },
}

pub async fn listen<F>(mut connection: Connection, is_client_side: bool, mut callback: F) where
F: FnMut(BearcubMessage) -> Option<BearcubMessage> {
    // The `Connection` lets us read/write redis **frames** instead of
    // byte streams. The `Connection` type is defined by mini-redis.
    let mut frame_buf:Vec<Frame> = vec![];
    let mut rem_frames: usize;

    loop {
        if let Some(frame_opt) = connection.read_frame().await.ok() {
            if frame_opt.is_some() {
                let frame = frame_opt.clone().unwrap();
                frame_buf.push(frame);

                let f = frame_opt.unwrap(); // our copy
                rem_frames = f.n_remaining_frames as usize;
                // println!("GOT: {:?}", frame_opt.unwrap());

                if rem_frames == 1 {
                    // This is the last frame
                    let my_frames = frame_buf;
                    frame_buf = vec![];
                    // This could probably be done nicer with generics, but I don't feel like
                    // dealing with it right now
                    let reply = if !is_client_side {
                        let this_msg = RequestMessage::from_frames(my_frames);
                        match this_msg {
                            Result::Ok(msg) => {
                                println!("got message: {:?}", &msg);
                                let response_msg = callback(BearcubMessage::Request { msg }).map(|x| match x {
                                    BearcubMessage::Request { .. } => ResponseMessage::Error { code: ERR_CODE_INVALID_MSG, description: ERR_DESC_INVALID_MSG.to_string() },
                                    BearcubMessage::Response { msg } => {
                                        msg
                                    },
                                });
                                println!("response message: {:?}", &response_msg);
                                response_msg.map(|z| BearcubMessage::Response { msg: z })
                            },
                            _ => {
                                // Write error back to user
                                println!("invalid message");
                                let resp_err = ResponseMessage::Error { code: ERR_CODE_INVALID_MSG, description: ERR_DESC_INVALID_MSG.to_string() };
                                Some(BearcubMessage::Response { msg: resp_err })
                            },
                        }
                    } else {
                        let this_msg = ResponseMessage::from_frames(my_frames);
                        match this_msg {
                            Result::Ok(msg) => {
                                println!("got message (client): {:?}", &msg);
                                let response_msg = callback(BearcubMessage::Response { msg }).map(|x| match x {
                                    BearcubMessage::Response { .. } => None,
                                    BearcubMessage::Request { msg } => {
                                        Some(msg)
                                    },
                                });
                                println!("response message: {:?}", &response_msg);
                                match response_msg {
                                    None => None,
                                    Some(x) => x.map(|z| BearcubMessage::Request { msg: z }),
                                }
                            },
                            _ => {
                                // Break out of everything 
                                None
                            },
                        }
                    };

                    if reply.is_none() {
                        break;
                    }

                    // Dispatch on type instead of generics also
                    let frames = match reply.unwrap() {
                        BearcubMessage::Request { msg } => msg.to_frames(),
                        BearcubMessage::Response { msg } => msg.to_frames(),
                    };
                    println!("got {} frames to write back...", frames.len());
                    let mut write_res: Result<usize> = Err(anyhow!("no frames to write"));
                    let mut fi: usize = 0;
                    'inner: loop {
                        if fi >= frames.len() {
                            break 'inner;
                        }
                        let f = &frames[fi];
                        write_res = connection.write_frame(f).await;
                        if !write_res.is_ok() {
                            break 'inner;
                        }
                        fi += 1;
                    }
                    match write_res {
                        Err(_z) => {
                            println!("client closed socket, breaking out...");
                            break;
                        },
                        _ => (),
                    }


                }
            } else {
                println!("got the else case");
                break;
            }
        }

    }
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

    pub async fn write_msg<T>(&mut self, msg: T) -> Result<()> where T: Frameable {
        let frames = msg.to_frames();
        for f in &frames {
            let z = self.write_frame(f).await;
        }
        Ok(())
    }
}

