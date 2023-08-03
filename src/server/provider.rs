use crate::storage::format::BlobNode;



pub struct Provider {
    data_dir: String,
    user_id: String,
    blob_root: Option<BlobNode>,
}

impl Provider {
    pub fn new(data_dir: String, user_id: String) -> Provider {
        Provider { data_dir, user_id, blob_root: None }
    }

    pub fn cheeck_root_structure(&mut self) {
        if self.blob_root.is_some() {
            return;
        }

        self.blob_root = BlobNode::from_file(&self.data_dir[..]).ok();
    }

    pub fn get_blob(&self, id: &str) -> Option<BlobNode> {
        None
    }

}

fn by_id_for_node(node: BlobNode, id: &str) -> Option<BlobNode> {
    None
}

