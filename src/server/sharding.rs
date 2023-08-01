
use std::collections::hash_map::DefaultHasher;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

pub struct ShardedMutexKvStore {
    data: Arc<Vec<Mutex<HashMap<String, Vec<u8>>>>>,
}


impl ShardedMutexKvStore {

    pub fn new(num_shards: usize) -> ShardedMutexKvStore {
        let mut db = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            db.push(Mutex::new(HashMap::new()));
        }
        ShardedMutexKvStore{data: Arc::new(db)}
    }

    fn hash(&self, key:&str) -> usize {
        let mut h = DefaultHasher::new();
        key.hash(&mut h);
        h.finish() as usize
    }
    
    pub fn get_shard(&self, user_id:String) -> usize {
        self.hash(&user_id[..]) % self.data.len()
    }
}

