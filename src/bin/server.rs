
use std::collections::HashMap;

use bearcub::{server::{connection::Connection, provider::UserProvider}, protocol::{wire::Frame, types::{RequestMessage, ResponseMessage, ERR_CODE_INVALID_MSG, ERR_DESC_INVALID_MSG}}, server::provider::Provider};
use tokio::net::{TcpListener, TcpStream};
use bytes::Bytes;
use anyhow::*;

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:9444").await.unwrap();
    println!("Waiting...");

    let user_provider = UserProvider::new("./data");

    loop {
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await.unwrap();
        let prov_clone = user_provider.clone();
        tokio::spawn(async move {
            process(socket, prov_clone).await;
        });
    }
}

async fn process(socket: TcpStream, user_provider: UserProvider) {
    // The `Connection` lets us read/write redis **frames** instead of
    // byte streams. The `Connection` type is defined by mini-redis.
    let mut connection = Connection::new(socket);
    let mut frame_buf:Vec<Frame> = vec![];
    let mut cur_uid:Option<String> = None;
    let mut rem_frames: usize;

    loop {
        if let Some(frame_opt) = connection.read_frame().await.ok() {
            if frame_opt.is_some() {
                let frame = frame_opt.clone().unwrap();
                frame_buf.push(frame);
                
                let f = frame_opt.unwrap(); // our copy
                if let Some(uid) = f.user_id {
                    cur_uid = Some(uid);
                }
                rem_frames = f.n_remaining_frames as usize;
                // println!("GOT: {:?}", frame_opt.unwrap());
                
                if rem_frames == 1 {
                    // This is the last frame
                    let my_frames = frame_buf;
                    frame_buf = vec![];
                    let prov = user_provider.get(&cur_uid.clone().unwrap()).unwrap();
                    // let _ = prov.get_skeleton_node("a"); // or whatever
                    let this_msg = RequestMessage::from_frames(my_frames);
                    let reply = match this_msg {
                        Result::Ok(msg) => {
                            println!("got message: {:?}", &msg);
                            let response_msg = prov.respond_to(msg).unwrap_or_else(|_| ResponseMessage::Error { code: ERR_CODE_INVALID_MSG, description: ERR_DESC_INVALID_MSG.to_string() });
                            response_msg
                        },
                        _ => {
                            // Write error back to user
                            let resp_err = ResponseMessage::Error { code: 2, description: "invalid message data".to_string() };
                            resp_err
                        },
                    };

                    let frames = reply.to_frames();
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
            }
        }

    }

}
