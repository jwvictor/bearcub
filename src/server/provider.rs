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


    // pub fn put_blob(&mut self, id: &str, title: &str, parent_id: Option<&str>) -> Result<()> {
    //     match parent_id {
    //         None => {
    //             match &mut self.skeleton {
    //                 Some(root) => {
    //                     root.add_child(BlobNode::new(id.to_string(), title.to_string(), vec![]));
    //                     Ok(())
    //                 },
    //                 None => Ok(())
    //             }
    //         },
    //         Some(pid) => {
    //             match &mut self.skeleton {
    //                 Some(root) => {
    //                     // let mut parent: &mut BlobNode = by_id_for_node(&root, pid).unwrap();
    //                     // parent.add_child(BlobNode::new(id.to_string(), title.to_string(), vec![]));
    //                     Ok(())
    //
    //                 },
    //                 None => Err(anyhow!("No skeleton loaded")),
    //             }
    //         }
    //     }
    // }
    //
    // pub fn get_blob(&self, id: &str) -> Option<BlobNodeRef> {
    //     match &self.skeleton {
    //         Some(root) => {
    //             let res = by_id_for_node(root.clone(), id);
    //             if res.is_some() {
    //                 Some(res.unwrap().clone())
    //             } else {
    //                 None
    //             }
    //         },
    //         None => None,
    //     }
    // }
}


// fn by_id_for_node<'a>(node: BlobNodeRef, id: &str) -> Option<BlobNodeRef> {
//     let n = node.node();
//     let node_borrow = (*n).borrow();
//     if node_borrow.id().eq(id) {
//         // let rig = node.clone();
//         Some(node)
//     } else {
//         println!("{} is not {}", node_borrow.id(), id);
//         let c = node_borrow.children();
//         for x in c {
//             let rv = by_id_for_node(x, id);
//             if rv.is_some() {
//                 // return Some(rv.unwrap().clone())
//                 return rv;
//             }
//         }
//         return None;
//     }
// }
//
//
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_get_by_id() {
//         const ID0: &str = "0";
//         const ROOT: &str = "root";
//         const ID1: &str = "1";
//         const TITLE1: &str = "notes";
//         const ID2: &str = "2";
//         const TITLE2: &str = "passwords";
//         let bc1 = BlobNode::new(ID1.to_string(), TITLE1.to_string(), vec![]);
//         let bc2 = BlobNode::new(ID2.to_string(), TITLE2.to_string(), vec![]);
//         let bc0 = BlobNode::new(ID0.to_string(), ROOT.to_string(), vec![bc1, bc2]);
//         let _ = bc0.flush_to_file("data/test1/blobs.bson");
//         let mut provider = Provider::new("./data".to_string(), "test1".to_string());
//         provider.check_root_structure();
//         let blob = provider.get_blob(ID1);
//         assert_eq!(blob.is_some(), true);
//         assert_eq!(blob.is_some(), true);
//
//
//     }
// }
