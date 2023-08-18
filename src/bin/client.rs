use bearcub::{server::{connection::{Connection, self}, self}, protocol::types::{RequestMessage, ResponseMessage}};
use bytes::Bytes;
use rand::Rng;
use tokio::net::TcpStream;
use uuid::Uuid;

// TODO - this thing was returning SUCCESS even when the parent ID didn't exist .. and it even
// saved the json blob to disk
async fn send_batch_messages(n: usize, user_id: String, conn: &mut Connection) {
    let mut first_id:Option<String> = None;
    for _i in 0..n {
        let id = Uuid::new_v4().to_string();
        let title:String = (0..12).map(|_i| {
            let x:u32 = rand::thread_rng().gen_range(97..122);
            char::from_u32(x).unwrap_or(char::from(30))
        }).collect();
        let s1 = "{\"title\": \"";
        let s2 = "\"}";
        let msg = RequestMessage::Put { user_id: user_id.clone(), id: id.clone(), parent: first_id.clone(), data: Bytes::from(format!("{}{}{}", s1, title, s2)) };
        println!("Batch message = {:?}", &msg);
        let z = conn.write_message(connection::BearcubMessage::Request { msg }).await;
        if first_id.is_none() {
            first_id = Some(id.clone());
        }
        if z.is_err() {
            println!("Error writing: {}", z.unwrap_err());
        } else {
            let res_frames = server::connection::read_one_message_frames(conn).await;
            if res_frames.is_ok() {
                let frs = res_frames.unwrap();
                let res_msg = ResponseMessage::from_frames(frs);
                println!("Got response message: {:?}", res_msg);
            } else {
                println!("Got return error: {:?}", res_frames.unwrap_err());

            }
        }
    }
}

async fn client_test(mut conn: Connection) {
    let mut ctr: usize = 0;


    send_batch_messages(1000, "beaa3a60-0082-4e5d-8153-a3c062dfdd2a".to_string(), &mut conn).await;

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
