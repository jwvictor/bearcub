use crate::storage::format::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use std::path::Path;
use std::fs::create_dir_all;
use anyhow::*;


#[derive(Debug,Clone)]
pub struct UserProvider {
    data_dir: String,
    providers: Arc<Mutex<RefCell<HashMap<String, Provider>>>>,
}

#[derive(Debug,Clone)]
pub struct Provider {
    data_dir: String,
    user_id: String,
    skeleton: Option<SkeletonHandleRef>,
}

impl UserProvider {
    pub fn new(data_dir: &str) -> UserProvider {
        UserProvider { providers: Arc::new(Mutex::new(RefCell::new(HashMap::new()))), data_dir: data_dir.to_string() }
    }

    // TODO - change this thing to use the sharded model prototyped in sharding.rs
    pub fn get(&self, uid: &str) -> Result<Provider> {
        let mg = self.providers.lock().unwrap();
        let mut borrow = mg.borrow_mut();
        if borrow.contains_key(uid) {
            let rig = borrow[uid].clone();
            Ok(rig)
        } else {
            let new_prov = Provider::new(self.data_dir.clone(), uid.to_string())?;
            let _ = borrow.insert(uid.to_string(), new_prov.clone());
            Ok(new_prov)
        }
    }
}

impl Provider {
    pub fn new(data_dir: String, user_id: String) -> Result<Provider> {
        let mut p = Provider { data_dir, user_id, skeleton: None };
        p.check_root_structure();
        p.check_storage_ready()?;
        Ok(p)
    }

    fn user_data_path(&self) -> String {
        let filename = format!("{}/{}", &self.data_dir, &self.user_id);
        filename
    }

    fn skeleton_filename(&self) -> String {
        let filename = format!("{}/blobs.bson", self.user_data_path());
        filename
    }

    pub fn check_root_structure(&mut self) {
        if self.skeleton.is_some() {
            return;
        }

        let filename = self.skeleton_filename();
        self.skeleton = SkeletonHandleRef::from_file(&filename[..]).ok();
    }

    pub fn check_storage_ready(&self) -> anyhow::Result<()> {
        let path = self.user_data_path();
        if !Path::new(&path[..]).exists() {
            let _ = create_dir_all(path)?;
            Ok(())
        }
        else {
            Ok(())
        }
    }

    pub fn flush(&mut self) -> Result<()> {
        self.check_root_structure();
        self.check_storage_ready()?;
        match &self.skeleton {
            Some(root) => root.flush_to_file(&self.skeleton_filename()[..]),
            None => Ok(()),
        }
    }

    pub fn get_skeleton_node(&self, id: &str) -> Option<SkeletonNode> {
        match &self.skeleton {
            Some(root) => root.get(id),
            None => None,
        }
    }
}
