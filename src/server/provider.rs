use crate::protocol::blobs::extract_title;
use crate::protocol::types::ERR_CODE_INVALID_MSG;
use crate::protocol::types::ERR_CODE_NO_SUCH_ENTITY;
use crate::protocol::types::ERR_DESC_INVALID_MSG;
use crate::protocol::types::ERR_DESC_NO_SUCH_ENTITY;
use crate::protocol::types::RequestMessage;
use crate::protocol::types::ResponseMessage;
use crate::storage::format::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use std::path::Path;
use std::fs::create_dir_all;
use anyhow::*;
use bytes::Bytes;


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
        p.check_storage_ready()?;
        p.check_root_structure();
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

    fn blob_filename(&self, id: &str) -> String {
        let filename = format!("{}/{}.json", self.user_data_path(), id);
        filename
    }

    pub fn load_blob_data(&self, id: &str) -> Result<Vec<u8>> {
        let s = std::fs::read(self.blob_filename(id))?;
        Ok(s)
    }

    pub fn write_blob_data(&self, id: &str, contents: &[u8]) -> Result<()> {
        std::fs::write(self.blob_filename(id), contents)?;
        Ok(())
    }

    pub fn check_root_structure(&mut self) {
        if self.skeleton.is_some() {
            return;
        }

        let filename = self.skeleton_filename();
        self.skeleton = SkeletonHandleRef::from_file(&filename[..]).ok();
        if self.skeleton.is_none() {
            // no such file exists
            println!("no file exists, creating empty structure...");
            self.skeleton = Some(SkeletonHandleRef::new());
        }
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
        self.check_storage_ready()?;
        self.check_root_structure();
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

    pub fn get_node_by_path(&self, path: &str) -> Option<SkeletonNode> {
        match &self.skeleton {
            Some(root) => root.get_by_path(path),
            None => None,
        }
    }

    pub fn set_node(&mut self, id: &str, data_bytes: Bytes) -> Result<()> {
        match &mut self.skeleton {
            Some(root) => {
                let title = extract_title(data_bytes.clone());
                if !title.is_ok() {
                    // println!("error extracting title: {:?}", title.unwrap_err());
                    return Err(anyhow!("no title found in data"));
                }
                let title_s = title.unwrap();
                let res1 = root.set_node(SkeletonNode::new(id, &title_s));
                if res1.is_err() {
                    println!("unable to set_node");
                    Err(anyhow!("unable to add node to structure"))
                } else {
                    println!("writing blob data for {:?}", &id);
                    self.write_blob_data(id, &data_bytes.to_vec()[..])
                }
            },
            _ => Err(anyhow!("no state loaded")),
        }
    }

    pub fn put_node(&mut self, id: &str, parent_id: Option<&str>, data_bytes: Bytes) -> Result<()> {
        match &mut self.skeleton {
            Some(root) => {
                let title = extract_title(data_bytes.clone());
                if !title.is_ok() {
                    // println!("error extracting title: {:?}", title.unwrap_err());
                    return Err(anyhow!("no title found in data"));
                }
                let title_s = title.unwrap();
                let x = root.add_node(SkeletonNode::new(id, &title_s[..]), parent_id);
                if !x.is_ok() {
                    return Err(anyhow!("error adding node to structure"));
                }
                let res1 = self.flush();
                let bs_vec = data_bytes.to_vec();
                let res2 = self.write_blob_data(id, &bs_vec[..]);
                if res1.is_err() || res2.is_err() {
                    Err(anyhow!("error writing to disk"))
                } else {
                    x
                }
            },
            None => Err(anyhow!("no state loaded")),
        }
    }

    pub fn respond_to(&mut self, request:RequestMessage) -> Result<ResponseMessage> {
        match request {
            RequestMessage::Get { user_id: _, id, path } => {
                if id.is_none() && path.is_none() {
                    return Ok(ResponseMessage::Error { code: ERR_CODE_NO_SUCH_ENTITY, description: ERR_DESC_NO_SUCH_ENTITY.to_string() })
                }
                let blob_id_opt = match id {
                    Some(ids) => Some(ids),
                    _ => {
                        let rel_node = self.get_node_by_path(&path.unwrap());
                        match rel_node {
                            Some(node) => Some(String::from(node.id())),
                            _ => None,
                        }
                        //
                    },
                };
                match blob_id_opt {
                    Some(ids) => {
                        let dat = self.load_blob_data(&ids)?;
                        Ok(ResponseMessage::Data { data: Bytes::from(dat) })
                    },
                    _ => Ok(ResponseMessage::Error { code: ERR_CODE_NO_SUCH_ENTITY, description: ERR_DESC_NO_SUCH_ENTITY.to_string() }),
                }
            },
            RequestMessage::Put { user_id: _, id, parent, data } => {
                let x = self.put_node(&id[..], parent.as_deref(), data);
                if x.is_ok() {
                    Ok(ResponseMessage::Data { data: Bytes::from("SUCCESS") })
                } else {
                    Ok(ResponseMessage::Error { code: ERR_CODE_INVALID_MSG, description: ERR_DESC_INVALID_MSG.to_string() })
                }
            },
            RequestMessage::Set { user_id: _, id, data } => {
                println!("Responding to set: {:?}", id.clone());
                self.set_node(&id[..], data)?;
                Ok(ResponseMessage::Data { data: Bytes::from("SUCCESS") })
            }
            _ => Ok(ResponseMessage::Error { code: ERR_CODE_INVALID_MSG, description: ERR_DESC_INVALID_MSG.to_string() }),
        }

    }
}
