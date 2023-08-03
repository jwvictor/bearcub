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

    fn skeleton_filename(&self) -> String {
        let filename = format!("{}/{}/blobs.bson", &self.data_dir, &self.user_id);
        filename

    }

    pub fn check_root_structure(&mut self) {
        if self.blob_root.is_some() {
            return;
        }

        let filename = self.skeleton_filename();
        self.blob_root = BlobNode::from_file(&filename[..]).ok();
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



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_by_id() {
        const ID0: &str = "0";
        const ROOT: &str = "root";
        const ID1: &str = "1";
        const TITLE1: &str = "notes";
        const ID2: &str = "2";
        const TITLE2: &str = "passwords";
        let bc1 = BlobNode::new(ID1.to_string(), TITLE1.to_string(), vec![]);
        let bc2 = BlobNode::new(ID2.to_string(), TITLE2.to_string(), vec![]);
        let bc0 = BlobNode::new(ID0.to_string(), ROOT.to_string(), vec![bc1, bc2]);
        let _ = bc0.flush_to_file("data/test1/blobs.bson");
        let mut provider = Provider::new("./data".to_string(), "test1".to_string());
        provider.check_root_structure();
        let blob = provider.get_blob(ID1);
        assert_eq!(blob.is_some(), true);
        assert_eq!(blob.is_some(), true);


    }
}
