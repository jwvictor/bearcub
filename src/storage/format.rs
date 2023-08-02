
use serde::{Serialize, Deserialize};
use anyhow::*;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use bson::*;

// Step 1: maintain a file with the hierarchy of blobs, to store in <user id>/blobs.bson
// Step 2: maintain individual files for blobs at <user id>/<blob id>.json


// The root blob node owns all child blobs.
#[derive(Debug, Serialize, Deserialize)]
pub struct BlobNode {
  id: String,
  title: String,
  children: Vec<BlobNode>,
}

impl BlobNode {
    pub fn new(id: String, title: String, children: Vec<BlobNode>) -> BlobNode {
        return BlobNode{id, title, children};
    }

    pub fn flush_to_file(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;
        let bs_obj = bson::to_bson(&self)?;
        let bs = bson::to_vec(&bs_obj)?;
        file.write_all(&bs[..])?;
        Ok(())
    }

    pub fn eq(&self, other: BlobNode) -> bool {
        self.id.eq(&other.id) && self.children.len() == other.children.len()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_flush() {
        const ID0: &str = "0";
        const ROOT: &str = "root";
        const ID1: &str = "1";
        const TITLE1: &str = "notes";
        const ID2: &str = "2";
        const TITLE2: &str = "passwords";
        let bc1 = BlobNode::new(ID1.to_string(), TITLE1.to_string(), vec![]);
        let bc2 = BlobNode::new(ID2.to_string(), TITLE2.to_string(), vec![]);
        let bc0 = BlobNode::new(ID0.to_string(), ROOT.to_string(), vec![bc1, bc2]);
        let res = bc0.flush_to_file("testfile.bson");
        assert_eq!(res.is_ok(), true);
        let input_file = fs::read("testfile.bson").unwrap();
        let deserialized: BlobNode = bson::from_slice_utf8_lossy(&input_file).unwrap();
        assert_eq!(deserialized.eq(bc0), true)
    }
}
