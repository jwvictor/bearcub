
use serde::{Serialize, Deserialize};
use anyhow::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use bson::*;




const ROOT_ID: &str = "root";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkeletonNode {
    id: String,
    title: String,
    child_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkeletonHandle {
    root: SkeletonNode,
    nodes: HashMap<String,SkeletonNode>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkeletonHandleRef {
    ptr: Arc<Mutex<RefCell<SkeletonHandle>>>,
}

impl SkeletonHandle {
    pub fn new() -> SkeletonHandle {
        let root = SkeletonNode { id: ROOT_ID.to_string(), title: ROOT_ID.to_string(), child_ids: vec![] };
        SkeletonHandle { root: root, nodes: HashMap::new() }
    }

    pub fn get(&self, id: &str) -> Option<SkeletonNode> {
        let rig = self.nodes.get(id);
        match rig {
            Some(x) => {
                let y = x.clone();
                Some(y)
            },
            None => None,
        }
    }

    pub fn set_node(&mut self, node: SkeletonNode) -> Result<()> {
        let _ = self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    pub fn add_node(&mut self, node: SkeletonNode, parent: Option<&str>) -> Result<()> {
        match parent {
            Some(pid) => {
                let borrow = self;
                let parent_node = borrow.nodes.get_mut(pid);
                if parent_node.is_some() {
                    parent_node.unwrap().add_child(&node.id[..]);
                }
                borrow.nodes.insert(node.id.clone(), node);
                Ok(())
            },
            None => {
                let selfb = self;
                selfb.root.add_child(&node.id[..]);
                let id_clone = node.id.clone();
                selfb.nodes.insert(id_clone, node);
                Ok(())
            },
        }
    }

    fn borrow(&self, id: &str) -> &SkeletonNode {
        self.nodes.get(id).unwrap()
    }

    fn get_by_path_parts(&self, from_id: &str, parts: Rc<Vec<&str>>, parts_idx: usize) -> Option<SkeletonNode> {
        let cur_node = self.borrow(from_id);
        if cur_node.title.starts_with(parts[parts_idx]) {
            // we have a match
            if (parts.len() - parts_idx) == 1 {
                return Some(cur_node.clone())
            } else {
                // recurse
                for cid in &cur_node.child_ids {
                    let res = self.get_by_path_parts(&cid[..], parts.clone(), parts_idx+1);
                    if res.is_some() {
                        return res;
                    }
                }
            }
        }
        None
    }

    fn top_level_ids(&self) -> Vec<String> {
        self.root.child_ids.clone()
    }

    pub fn get_by_path(&self, path: &str) -> Option<SkeletonNode> {
        let parts: Vec<&str> = path.split(":").collect();
        let tli = self.top_level_ids();
        let rc_parts = Rc::new(parts);
        for n in tli {
            let res = self.get_by_path_parts(&n[..], rc_parts.clone(), 0);
            if res.is_some() {
                return res;
            }
        }
        None
    }

}

impl SkeletonHandleRef {

    pub fn new() -> SkeletonHandleRef {
        SkeletonHandleRef { ptr: Arc::new(Mutex::new(RefCell::new(SkeletonHandle::new()))) }
    }

    pub fn top_level_ids(&self) -> Vec<String> {
        self.ptr.lock().unwrap().borrow().top_level_ids()
    }

    pub fn get(&self, id: &str) -> Option<SkeletonNode> {
        self.ptr.lock().unwrap().borrow().get(id)
    }

    pub fn get_by_path(&self, id: &str) -> Option<SkeletonNode> {
        self.ptr.lock().unwrap().borrow().get_by_path(id)
    }

    pub fn set_node(&mut self, node: SkeletonNode) -> Result<()> {
        let guard = self.ptr.lock().unwrap();
        let mut borrow = guard.borrow_mut();
        borrow.set_node(node)
    }

    pub fn add_node(&mut self, node: SkeletonNode, parent: Option<&str>) -> Result<()> {
        let guard = self.ptr.lock().unwrap();
        let mut borrow = guard.borrow_mut();
        borrow.add_node(node, parent)
    }

    pub fn flush_to_file(&self, path: &str) -> Result<()> {
        let bs_obj = bson::to_bson(&self)?;
        let mut file = File::create(path)?;
        let bs = bson::to_vec(&bs_obj)?;
        file.write_all(&bs[..])?;
        Ok(())
    }

    pub fn from_file(path: &str) -> Result<SkeletonHandleRef> {
        let fp = fs::read(path)?;
        let deser: SkeletonHandleRef = bson::from_slice(&fp[..])?;
        Ok(deser)
    }
}

impl SkeletonNode {
    pub fn new(id: &str, title: &str) -> SkeletonNode {
        SkeletonNode { id: id.to_string(), title: title.to_string(), child_ids: vec![] }
    }
    pub fn add_child(&mut self, id: &str) {
        self.child_ids.push(id.to_string())
    }
    pub fn id(&self) -> &str {
        &self.id[..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_skeleton() {
        // let mut h = SkeletonHandle::new();
        let mut h = SkeletonHandleRef::new();
        let n1 = SkeletonNode::new("n1", "top-level-node");
        let _ = h.add_node(n1, None);
        let tl = h.top_level_ids();
        assert_eq!(tl.len(), 1);
        let n2 = SkeletonNode::new("n2", "child-node");
        let _ = h.add_node(n2, Some("n1"));
        let tl2 = h.top_level_ids();
        assert_eq!(tl2.len(), 1);
        let gn1 = h.get("n2").unwrap();
        assert_eq!(gn1.title.eq("child-node"), true);
        let gp1 = h.get_by_path("top").unwrap();
        assert_eq!(gp1.id.eq("n1"), true);
        let gp2 = h.get_by_path("top:chi").unwrap();
        assert_eq!(gp2.id.eq("n2"), true);
        let f_res = h.flush_to_file("testfile.bson");
        assert_eq!(f_res.is_ok(), true);

        let h2 = SkeletonHandleRef::from_file("testfile.bson").unwrap();
        let h2_tlis = h2.top_level_ids();
        assert_eq!(h2_tlis.len(), 1);
        assert_eq!(h2_tlis[0], "n1");
    }
}

