use bearcub::{server::{connection::{Connection, self}, provider::UserProvider}, protocol::types::{ResponseMessage, ERR_CODE_INVALID_MSG, ERR_DESC_INVALID_MSG, ERR_CODE_NO_SUCH_ENTITY, ERR_DESC_NO_SUCH_ENTITY}};
use tokio::net::{TcpListener, TcpStream};

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
            // process(socket, prov_clone).await;
            listen(socket, prov_clone).await;
        });
    }
}

async fn listen(socket: TcpStream, user_provider: UserProvider) {
    connection::listen(Connection::new(socket), false, |x| {
        match x {
            connection::BearcubMessage::Request { msg } => {
                let cur_uid = msg.user_id().unwrap_or_else(|| String::new());
                let prov = user_provider.get(&cur_uid[..]);
                if prov.is_err() {
                    Some(connection::BearcubMessage::Response { msg: ResponseMessage::Error { code: ERR_CODE_NO_SUCH_ENTITY, description: ERR_DESC_NO_SUCH_ENTITY.to_string() } })
                } else {
                    let res_msg = prov.unwrap().respond_to(msg).unwrap_or_else(|e| { 
                        println!("Got error processing message: {:?}", e);
                        ResponseMessage::Error { code: ERR_CODE_INVALID_MSG, description: ERR_DESC_INVALID_MSG.to_string() } 
                    });
                    Some(connection::BearcubMessage::Response { msg: res_msg })
                }
            },
            _ => None,
        }
    }).await;
}
