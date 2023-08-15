use std::net::SocketAddr;

use bearcub::{say_hello, server::connection::{Connection, self}, protocol::{wire::Frame, types::{RequestMessage, ResponseMessage}}};
use bytes::Bytes;
use tokio::{net::TcpStream, io::AsyncWriteExt};
use anyhow::*;

async fn receive_message(mut conn: Connection) -> Result<Option<ResponseMessage>> {

    let mut frame_buf:Vec<Frame> = vec![];
    let mut rem_frames: usize;

    Ok(None)
}

async fn client_test(mut conn: Connection) {
    let mut ctr: usize = 0;

    // Write the intitial message
    let msg = RequestMessage::Put { user_id: "beaa3a60-0082-4e5d-8153-a3c062dfdd2a".to_string(), id: "0e58d858-0808-4cef-8143-8eb4db188a64".to_string(), parent: None, data: Bytes::from("{\"title\": \"abc\"}") };
    let frames = msg.to_frames();
    for frame in &frames {
        let res = conn.write_frame(frame).await;
        if res.is_err() {
            break;
        }
    }

    // and listen for subsequent messages
    connection::listen(conn, true, |x| {
        ctr += 1;
        println!("In client_test closure (ctr {})", ctr);
        match x {
            connection::BearcubMessage::Response { msg } => {
                println!("Got msg: {:?}", &msg);
                if ctr > 2 { None } else { Some(connection::BearcubMessage::Request { msg: RequestMessage::Put { user_id: "beaa3a60-0082-4e5d-8153-a3c062dfdd2a".to_string(), id: "0e58d858-0808-4cef-8143-8eb4db188a64".to_string(), parent: None, data: Bytes::from("{\"title\": \"abc\"}") }}) }
            },
            _ => None,
        }
    }).await;
}

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:9444").await.unwrap();
    let mut conn = Connection::new(stream);
    client_test(conn).await;
    // let msg = RequestMessage::Put { user_id: "beaa3a60-0082-4e5d-8153-a3c062dfdd2a".to_string(), id: "0e58d858-0808-4cef-8143-8eb4db188a64".to_string(), parent: None, data: Bytes::from("{\"title\": \"abc\"}") };
    // let frames = msg.to_frames();
    // for frame in &frames {
    //     let res = conn.write_frame(frame).await;
    //     if res.is_err() {
    //         break;
    //     }
    // }
}
