use bearcub::{server::connection::{Connection, self}, protocol::types::RequestMessage};
use bytes::Bytes;
use tokio::net::TcpStream;
use uuid::Uuid;

async fn send_batch_messages(n: usize, user_id: String) {
    let mut first_id:Option<String> = None;
    for i in 0..n {
        let id = Uuid::new_v4().to_string();
        if first_id.is_none() {
            first_id = Some(id.clone());
        }
    let msg = RequestMessage::Put { user_id: "beaa3a60-0082-4e5d-8153-a3c062dfdd2a".to_string(), id: "0e58d858-0808-4cef-8143-8eb4db188a64".to_string(), parent: None, data: Bytes::from("{\"title\": \"abc\"}") };

    }
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
                println!("Got msg from server: {:?}", &msg);
                if ctr > 5 {
                    // Hang up
                    None 
                } else if ctr == 5 { 
                    println!("\t DOING LIST");
                    Some(connection::BearcubMessage::Request { msg: RequestMessage::List { user_id: "beaa3a60-0082-4e5d-8153-a3c062dfdd2a".to_string(), blob_id: None }}) 
                } else if ctr == 4 { 
                    println!("\t DOING REQ PATH - CORRECT TITLE");
                    Some(connection::BearcubMessage::Request { msg: RequestMessage::Get { user_id: "beaa3a60-0082-4e5d-8153-a3c062dfdd2a".to_string(), path: Some("def".to_string()), id: None }}) 
                } else if ctr == 3 { 
                    println!("\t DOING REQ PATH - OLD TITLE");
                    Some(connection::BearcubMessage::Request { msg: RequestMessage::Get { user_id: "beaa3a60-0082-4e5d-8153-a3c062dfdd2a".to_string(), path: Some("abc".to_string()), id: None }}) 
                } else if ctr == 2 {
                    println!("\t DOING SET");
                    Some(connection::BearcubMessage::Request { msg: RequestMessage::Set { user_id: "beaa3a60-0082-4e5d-8153-a3c062dfdd2a".to_string(), id: "0e58d858-0808-4cef-8143-8eb4db188a64".to_string(), data: Bytes::from("{\"title\": \"def\"}") }}) 
                } else { 
                    println!("\t DOING REQ ID");
                    Some(connection::BearcubMessage::Request { msg: RequestMessage::Get { user_id: "beaa3a60-0082-4e5d-8153-a3c062dfdd2a".to_string(), id: Some("0e58d858-0808-4cef-8143-8eb4db188a64".to_string()), path: None }}) 
                }
            },
            _ => None,
        }
    }).await;
}

#[tokio::main]
async fn main() {
    let stream = TcpStream::connect("127.0.0.1:9444").await.unwrap();
    let conn = Connection::new(stream);
    client_test(conn).await;
}
