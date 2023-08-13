use std::net::SocketAddr;

use bearcub::{say_hello, server::connection::Connection, protocol::{wire::Frame, types::RequestMessage}};
use bytes::Bytes;
use tokio::{net::TcpStream, io::AsyncWriteExt};
use anyhow::*;


#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:9444").await.unwrap();

    // Write some data.
    // stream.write_all(b"hello world!").await;
    let mut conn = Connection::new(stream);
    let msg = RequestMessage::Put { user_id: "beaa3a60-0082-4e5d-8153-a3c062dfdd2a".to_string(), id: "0e58d858-0808-4cef-8143-8eb4db188a64".to_string(), parent: None, data: Bytes::from("{\"title\": \"abc\"}") };
    let frames = msg.to_frames();
    for frame in &frames {
        let res = conn.write_frame(frame).await;
        if res.is_err() {
            break;
        }
    }
    // let frame = Frame::new(Some("e17ca57f-a8db-4a0d-b9a9-6ff9edc983fd".to_string()), 1 as u32, 'G' as u8, Bytes::from_static(b"abc"));
    // conn.write_frame(&frame).await;
    // conn.write_frame(&frame).await;
    let z = conn.read_frame().await.unwrap();
    loop {
        let read_res = conn.read_frame().await;
        if let Some(frame) = read_res.ok() {
            match frame {
                Some(data) => {
                    println!("GOT FRAME: {:?}", data);
                },
                None => break,
            }
        }
    }
    /*let addr_str = "localhost:6739";
    let addr = addr_str.parse::<SocketAddr>().unwrap();
    let tcp_conn = TcpStream::connect(addr).await.unwrap();
    tcp_conn.writable().await.unwrap();
    tcp_conn.try_write(b"abc");*/
}
