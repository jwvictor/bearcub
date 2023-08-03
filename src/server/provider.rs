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
        match &self.blob_root {
            Some(root) => {
                //
                let res = by_id_for_node(root, id);
                res
            },
            None => None,
        }
    }

}

fn by_id_for_node(node: &BlobNode, id: &str) -> Option<BlobNode> {
    if node.id().eq(id) {
        let rig = node.clone();
        Some(rig)
    } else {
        let c = node.children();
        for x in c {
            let rv = by_id_for_node(x, id);
            if rv.is_some() {
                // return Some(rv.unwrap().clone())
                return rv;
            }
        }
        return None;
    }
}

