
use serde::{Serialize, Deserialize};
use anyhow::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;
use bson::*;




const ROOT_ID: &str = "root";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkeletonNode {
    id: String,
    title: String,
    child_ids: Vec<String>,
}

pub struct SkeletonHandle {
    root: Mutex<RefCell<SkeletonNode>>,
    nodes: Mutex<HashMap<String,SkeletonNode>>,
}

pub struct SkeletonHandleRef {
    ptr: Arc<SkeletonHandle>,
}

impl SkeletonHandle {
    pub fn new() -> SkeletonHandle {
        let root = SkeletonNode { id: ROOT_ID.to_string(), title: ROOT_ID.to_string(), child_ids: vec![] };
        SkeletonHandle { root: Mutex::new(RefCell::new(root)), nodes: Mutex::new(HashMap::new()) }
    }

    fn top_level_ids(&self) -> Vec<String> {
        self.root.lock().unwrap().borrow().child_ids.clone()
    }

    pub fn add_node(&mut self, node: SkeletonNode, parent: Option<&str>) -> Result<()> {
        match parent {
            Some(pid) => {
                let mut hm = self.nodes.lock().unwrap();
                let parent_node = hm.get_mut(pid);
                if parent_node.is_some() {
                    parent_node.unwrap().add_child(pid);
                }
                Ok(())
            },
            None => {
                let root = self.root.lock();
                root.unwrap().borrow_mut().add_child(&node.id[..]);
                let mut hm = self.nodes.lock().unwrap();
                let id_clone = node.id.clone();
                hm.insert(id_clone, node);
                Ok(())
            },
        }
    }

}

impl SkeletonNode {
    pub fn new(id: &str, title: &str) -> SkeletonNode {
        SkeletonNode { id: id.to_string(), title: title.to_string(), child_ids: vec![] }
    }
    pub fn add_child(&mut self, id: &str) {
        self.child_ids.push(id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_skeleton() {
        let mut h = SkeletonHandle::new();
        let n1 = SkeletonNode::new("n1", "top-level-node");
        h.add_node(n1, None);
        let tl = h.top_level_ids();
        assert_eq!(tl.len(), 1);
        let n2 = SkeletonNode::new("n2", "child-node");
        h.add_node(n2, Some("n1"));
        let tl2 = h.top_level_ids();
        assert_eq!(tl2.len(), 1);
    }
}



// Step 1: maintain a file with the hierarchy of blobs, to store in <user id>/blobs.bson
// Step 2: maintain individual files for blobs at <user id>/<blob id>.json

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct BlobNodeRef {
//     node: Rc<RefCell<BlobNode>>,
// }
//
//
// // The root blob node owns all child blobs.
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct BlobNode {
//   id: String,
//   title: String,
//   children: Vec<BlobNodeRef>,
// }
//
// impl BlobNode {
//     pub fn new(id: String, title: String, children: Vec<BlobNode>) -> BlobNode {
//         let children_refs:Vec<BlobNodeRef> = children.into_iter().map(|x| BlobNodeRef{node: Rc::new(RefCell::new(x))}).collect();
//         return BlobNode{id, title, children: children_refs};
//     }
//
//     pub fn id(&self) -> &str {
//         &self.id[..]
//     }
//
//     pub fn from_file(path: &str) -> Result<BlobNodeRef> {
//         // let fpath = path.to_string();
//         let fp = fs::read(path)?;
//         let deser: BlobNodeRef = bson::from_slice(&fp[..])?;
//         Ok(deser)
//
//     }
//
//     pub fn children(&self) -> Vec<BlobNodeRef> {
//         let mut v = vec![];
//         for c in &self.children {
//             v.push(c.clone());
//         }
//         v
//     }
//
//     pub fn add_child(&mut self, child: BlobNode) {
//         self.children.push(BlobNodeRef{ node: Rc::new(RefCell::new(child))});
//     }
//
//     pub fn flush_to_file(&self, path: &str) -> Result<()> {
//         let mut file = File::create(path)?;
//         let bs_obj = bson::to_bson(&self)?;
//         let bs = bson::to_vec(&bs_obj)?;
//         file.write_all(&bs[..])?;
//         Ok(())
//     }
//
//     pub fn eq(&self, other: BlobNode) -> bool {
//         self.id.eq(&other.id) && self.children.len() == other.children.len()
//     }
// }
//
// impl BlobNodeRef {
//     pub fn from(node: BlobNode) -> BlobNodeRef {
//         BlobNodeRef{ node: Rc::new(RefCell::new(node))}
//     }
//
//     pub fn flush_to_file(&self, path: &str) -> Result<()> {
//         let x = (*self.node).borrow();
//         x.flush_to_file(path)
//     }
//    
//
//     // TODO - everything is going to explode due to the runtime borrow checking of these ref cells.
//     // need to enforce concurrent access in some sane-ish way
//     pub fn add_child(&self, child: BlobNode) {
//         let mut root = (*self.node).borrow_mut();
//         root.add_child(child)
//     }
//
//     pub fn node(&self) -> Rc<RefCell<BlobNode>> {
//         self.node.clone()
//     }
//
// }
//
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_node_flush() {
//         const ID0: &str = "0";
//         const ROOT: &str = "root";
//         const ID1: &str = "1";
//         const TITLE1: &str = "notes";
//         const ID2: &str = "2";
//         const TITLE2: &str = "passwords";
//         let bc1 = BlobNode::new(ID1.to_string(), TITLE1.to_string(), vec![]);
//         let bc2 = BlobNode::new(ID2.to_string(), TITLE2.to_string(), vec![]);
//         let bc0 = BlobNode::new(ID0.to_string(), ROOT.to_string(), vec![bc1, bc2]);
//         let res = bc0.flush_to_file("testfile.bson");
//         assert_eq!(res.is_ok(), true);
//         let input_file = fs::read("testfile.bson").unwrap();
//         let deserialized: BlobNode = bson::from_slice_utf8_lossy(&input_file).unwrap();
//         assert_eq!(deserialized.eq(bc0), true)
//     }
// }
