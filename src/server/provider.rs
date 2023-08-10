use crate::storage::format::*;
use std::path::Path;
use std::fs::create_dir_all;
use anyhow::*;

pub struct Provider {
    data_dir: String,
    user_id: String,
    skeleton: Option<SkeletonHandleRef>,
}

impl Provider {
    pub fn new(data_dir: String, user_id: String) -> Provider {
        Provider { data_dir, user_id, skeleton: None }
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
