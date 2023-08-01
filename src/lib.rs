pub mod protocol {
    pub mod types;
    pub mod wire;
}

pub mod server {
    pub mod connection;
    pub mod sharding;
}

pub fn say_hello() {
    println!("Hello, world!");
}
