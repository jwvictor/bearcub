
use bearcub::{server::connection::Connection, protocol::wire::Frame};
use tokio::net::{TcpListener, TcpStream};
use bytes::Bytes;

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:9444").await.unwrap();
    println!("Waiting...");

    loop {
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    // The `Connection` lets us read/write redis **frames** instead of
    // byte streams. The `Connection` type is defined by mini-redis.
    let mut connection = Connection::new(socket);

    loop {
        if let Some(frame) = connection.read_frame().await.ok() {
            println!("GOT: {:?}", frame);

            // Respond with an error
            let response = Frame::new(None, 0 as u32, 'd' as u8, Bytes::new());
            let write_res = connection.write_frame(&response).await;
            if !write_res.is_ok() {
                println!("client closed socket, breaking out...");
                break;
            }
        }

    }

}
