use bytes::{Bytes, BytesMut, BufMut};
use std::io::{Cursor, Read};

#[derive(Debug, Clone)]
pub struct Frame {
    pub user_id: Option<String>,
    pub n_remaining_frames: u32,
    pub msg_type_flag: u8,
    pub data: Bytes,
}

// Codec implemented here
impl Frame {

    pub fn new(user_id: Option<String>, n_remaining_frames:u32, msg_type_flag:u8, data: Bytes) -> Frame {
        let f = Frame{
            user_id,
            n_remaining_frames, 
            msg_type_flag, 
            data, 
        };
        f
    }

    pub fn size(&self) -> usize {
        let mut prefix_sz = 4 + 4 + 4 + 1;
        if is_user_id_required_msgtype(self.msg_type_flag) {
            prefix_sz += 36;
        }
        prefix_sz + self.data.len()
    }

    pub fn to_bytes(&self) -> Bytes {
        let total_sz = self.size();
        let mut bs = BytesMut::with_capacity(self.size());
        bs.put_slice("c0.1".as_bytes());
        bs.put_u32(total_sz as u32);
        bs.put_u32(self.n_remaining_frames);
        bs.put_u8(self.msg_type_flag);
        if is_user_id_required_msgtype(self.msg_type_flag) {
            if self.user_id.is_none() {
                panic!("Get, Put, Set requests require user_id field");
            }
        }
        match &self.user_id {
            Some(uid) => {
                bs.put_slice(&uid[..].as_bytes());
                ()
            },
            _ => (),
        }
        bs.put(&self.data[..]);
        println!("to_bytes: {:?}", bs);
        bs.freeze()
    }
}

pub fn is_user_id_required_msgtype(msg_type_flag:u8) -> bool {
    let user_id_req:Vec<u8> = vec!['G', 'L', 'P', 'p', 's', 'R'].into_iter().map(|x| x as u8).collect();
    user_id_req.contains(&msg_type_flag)
}

pub fn check_frame(buf:&mut Cursor<&[u8]>, buf_len: usize) -> Option<usize> {
    if buf_len < (13 as usize) {
        println!("buffer too short to read header");
        None
    } else {
        buf.set_position(4);
        let mut sz4:[u8; 4] = [0; 4];
        match buf.read_exact(&mut sz4) {
            Ok(_) => {
                let sz = u32::from_be_bytes(sz4);
                println!("message size is {}, need at least a buffer that long (buffer is {})...", (sz as usize), buf_len);
                if buf_len >= (sz as usize) {
                println!("returning size {}, (buffer is {})...", (sz as usize), buf_len);
                    Some(sz as usize)
                } else {
                    None
                }
            },
            _ => None
        }
    }
}

pub fn try_parse_frame(buf: &mut Cursor<&[u8]>, buf_len: usize) -> Option<Frame> {
    if buf_len < (13 as usize) {
        // header not yet received
        None
    } else {
        let mut sz4:[u8; 4] = [0; 4];
        buf.read_exact(&mut sz4).ok();
        let version_string = String::from_utf8(sz4.to_vec());
        let bvs = version_string.clone().unwrap_or_default();
        if !bvs.eq("c0.1") {
            // let mut buf2 = buf.clone();
            // let mut s2 = String::new();
            // buf2.read_to_string(&mut s2);
            // println!("bad v string: {:?} / {:?} / {:?}", sz4, version_string.clone().unwrap_or_default(), s2);
            return None
        }
        
        buf.read_exact(&mut sz4).ok();
        let sz = u32::from_be_bytes(sz4);

        if buf_len < ((sz as usize) - 0) {
            println!("insufficient bytes: {} vs {}", buf_len, sz);
            return None
        }
        
        buf.read_exact(&mut sz4).ok();
        let n_remaining_frames = u32::from_be_bytes(sz4);

        let bytes_to_read = (sz - 13) as usize;
        let mut rem_buf = vec![0u8; bytes_to_read + 1];
        buf.read_exact(&mut rem_buf).ok()?;
        let msg_type_flag = rem_buf[0];

        let user_id: Option<String> = if is_user_id_required_msgtype(msg_type_flag){
            Some(String::from_utf8(rem_buf[1..37].to_vec()).ok()?)
        } else {
            None
        };
        let data_start_idx: usize = if is_user_id_required_msgtype(msg_type_flag) { 37 } else { 1 };
        let mut frame_bytes = BytesMut::with_capacity(rem_buf.len());
        frame_bytes.put_slice(&rem_buf[data_start_idx..]);
        let f = Frame::new(user_id, n_remaining_frames, msg_type_flag, frame_bytes.freeze());
        Some(f)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use bytes::Buf;

    #[test]
    fn test_frame_deserialization() {
        let uuid = "e17ca57f-a8db-4a0d-b9a9-6ff9edc983fd";
        let data = Bytes::from("hello".as_bytes());
        let f = Frame::new(Some(uuid.to_string()), 1, 'G' as u8, data);
        let bs = f.to_bytes();
        let mut bs_buffer = BytesMut::with_capacity(bs.len());
        bs_buffer.put(bs);
        let mut buf = Cursor::new(&bs_buffer[..]);
        let sz_opt = check_frame(&mut buf, bs_buffer.len());
        assert_eq!(sz_opt.is_some(), true);
        let frame_sz = sz_opt.unwrap();
        buf.set_position(0);
        let frame_opt = try_parse_frame(&mut buf, frame_sz);
        assert_eq!(frame_opt.is_some(), true);
        let frame = frame_opt.unwrap();
        assert_eq!(frame.n_remaining_frames, 1);
        assert_eq!(frame.size(), 5 + 13 + 36);
        assert_eq!(frame.msg_type_flag, 'G' as u8);
    }

    #[test]
    fn test_frame_serialization() {
        let uuid = "e17ca57f-a8db-4a0d-b9a9-6ff9edc983fd";
        let data = Bytes::from("hello".as_bytes());
        let f = Frame::new(Some(uuid.to_string()), 1, 'G' as u8, data);
        assert_eq!(f.size(), 18+36);
        let mut bs = f.to_bytes();
        
        let mut v_bs = bs.split_to(4);
        let v_str = String::from_utf8(v_bs.to_vec()).unwrap();
        assert_eq!(v_str.eq("c0.1"), true);

        v_bs = bs.split_to(4);
        println!("v_bs = {:?}", &v_bs[..]);
        let mut sz4:[u8; 4] = [0; 4];
        v_bs.copy_to_slice(&mut sz4);
        println!("sz_4 = {:?}", &sz4[..]);
        let sz = u32::from_be_bytes(sz4);
        assert_eq!(sz, 18+36);

        let _ = bs.split_to(4);
        
        v_bs = bs.split_to(1);
        
        assert_eq!(v_bs[0], 'G' as u8);
        
        let _ = bs.split_to(36);

        let dat_str = String::from_utf8(bs.to_vec()).unwrap();
        assert_eq!(dat_str.eq("hello"), true);
    }

    #[test]
    fn test_user_id_required_helper() {
        assert_eq!(is_user_id_required_msgtype('G' as u8), true);
        assert_eq!(is_user_id_required_msgtype('d' as u8), false);
    }
}
