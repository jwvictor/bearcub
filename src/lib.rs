pub mod protocol {
    pub mod types;
    pub mod wire;
    pub mod blobs;
}

pub mod server {
    pub mod connection;
    pub mod sharding;
    pub mod provider;
}

pub mod storage {
    pub mod format;
}

pub fn say_hello() {
    println!("Hello, world!");
}
