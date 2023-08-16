use anyhow::anyhow;
use bytes::{BytesMut, Bytes, BufMut, Buf};
use anyhow::*;
use super::wire::Frame;

// TODO: implement protocol stuff for `user_id` field

#[derive(Debug)]
pub enum RequestMessage {
    Get {
        user_id: String,
        id: Option<String>,
        path: Option<String>,
    },
    Put {
        user_id: String,
        id: String,
        parent: Option<String>,
        data: Bytes,
    },
    Set {
        user_id: String,
        id: String,
        data: Bytes,
    },
    Remove {
        user_id: String,
        id: String,
    },
}

#[derive(Debug)]
pub enum ResponseMessage {
    Error {
        code: u32,
        description: String,
    },
    Data {
        data: Bytes,
    }
}
pub const BUF_CAP: usize = 4096;
pub const BUF_CAP_HEADER_SZ_RES: usize = 128;
pub const DATA_BYTES_PER_FRAME : usize = BUF_CAP - BUF_CAP_HEADER_SZ_RES;

// Error codes
pub const ERR_CODE_INVALID_MSG: u32 = 11;
pub const ERR_DESC_INVALID_MSG: &'static str = "invalid message";
pub const ERR_CODE_NO_SUCH_ENTITY: u32 = 12;
pub const ERR_DESC_NO_SUCH_ENTITY: &'static str = "no such entity";

impl ResponseMessage {
    pub fn to_frames(mut self) -> Vec<Frame> {
        match &mut self {
            ResponseMessage::Error{code, description} => {
                let mut buf = BytesMut::with_capacity(4 + description.len());
                buf.put_u32(*code);
                buf.put_slice(description.as_bytes());
                vec![Frame::new(None, 1, 'e' as u8, buf.freeze())]
            },
            Self::Data{data} => {
                let bytes_per_frame = DATA_BYTES_PER_FRAME;
                let mut n_frames = data.len() / bytes_per_frame;
                if data.len() % bytes_per_frame != 0 {
                    n_frames += 1;
                }

                let mut frames:Vec<Frame> = vec![];
                let mut bs_remaining = data.len();

                let mut frame_idx = 0;
                while bs_remaining > 0 {
                    let bs_to_read = bs_remaining.min(bytes_per_frame);
                    let fr_dat = data.split_to(bs_to_read);
                    let f = Frame::new(None, (n_frames - frame_idx) as u32, 'd' as u8, fr_dat); 
                    bs_remaining -= bs_to_read;
                    frame_idx += 1;
                    frames.push(f);
                }
                frames
            },
        }
    }

    fn from_frames_data(frames: Vec<Frame>) -> Bytes {
        let mut out = BytesMut::with_capacity(BUF_CAP*frames.len());
        for f in frames {
            let bs = f.data.to_vec();
            out.put_slice(&bs[..]);
        }
        out.freeze()
    }

    pub fn from_frames(frames: Vec<Frame>) -> Result<ResponseMessage> {
        let mut f0 = frames[0].clone();
        match f0.msg_type_flag {
            b'd' => {
                Ok(ResponseMessage::Data { data: Self::from_frames_data(frames) })
            },
            b'e' => {
                // error
                let mut rig = f0.data.split_to(4);
                let code = rig.get_u32();
                let desc:Vec<u8> = f0.data.to_vec();
                Ok(ResponseMessage::Error { code, description: String::from_utf8(desc).unwrap_or(String::from("unknown error")) })
            },
            _ => Err(anyhow!("invalid msg type flag")),
        }
    }
}

impl RequestMessage {

    fn from_frames_getbyid(f: Frame) -> Result<RequestMessage> {
        // the bytes are just the ID
        let id_s = String::from_utf8(f.data.to_vec())?;
        match f.user_id {
            Some(uid) => 
                Ok(RequestMessage::Get { user_id: uid, id: Some(id_s), path: None }),
            _ => Err(anyhow!("rig"))
        }
    }

    fn from_frames_getbypath(f: Frame) -> Result<RequestMessage> {
        // the bytes are just the ID
        let path_s = String::from_utf8(f.data.to_vec())?;
        match f.user_id {
            Some(uid) => 
                Ok(RequestMessage::Get { user_id: uid, path: Some(path_s), id: None }),
            _ => Err(anyhow!("rig"))
        }
    }

    fn from_frames_put_set(frames: Vec<Frame>, is_set: bool) -> Result<RequestMessage> {
        let mut data = BytesMut::with_capacity(frames.len() * BUF_CAP);
        let mut f0 = frames[0].clone();
        let id_bytes = f0.data.split_to(36);
        let pid_bytes = if !is_set { Some(f0.data.split_to(36)) } else { None };
        let id_s = String::from_utf8(id_bytes.to_vec()).unwrap_or_else(|_| String::new());
        // TODO: restructure this hideous thing
        let pid_opt = if pid_bytes.is_none() { None } else { if pid_bytes.clone().unwrap().to_vec().iter().map(|x| *x != 0 as u8).reduce(|x,y| x || y).unwrap_or(false) { Some(String::from_utf8(pid_bytes.unwrap().to_vec()).ok().unwrap_or_else(|| String::new())) } else { None } };
        data.put(f0.data.split_to(f0.data.len()));
        for i in 1..frames.len() {
            data.put(frames[i].data.clone());
        }
        let fmsg = if !is_set { 
            RequestMessage::Put { user_id: f0.user_id.unwrap_or_else(|| String::new()), id: id_s, parent: pid_opt, data: data.freeze() } 
        } else {
            RequestMessage::Set { user_id: f0.user_id.unwrap_or_else(|| String::new()), id: id_s, data: data.freeze() }
        };
        Ok(fmsg)
    }

    fn from_frames_set(frames: Vec<Frame>) -> Result<RequestMessage> {
        Self::from_frames_put_set(frames, true)
    }
    fn from_frames_put(frames: Vec<Frame>) -> Result<RequestMessage> {
        Self::from_frames_put_set(frames, false)
    }

    pub fn from_frames(frames: Vec<Frame>) -> Result<RequestMessage> {
        let f0 = frames[0].clone();
        match f0.msg_type_flag {
            b'G' => RequestMessage::from_frames_getbyid(f0),
            b'P' => RequestMessage::from_frames_getbypath(f0),
            b'p' => RequestMessage::from_frames_put(frames),
            b's' => RequestMessage::from_frames_set(frames),
            _ => Err(anyhow!("invalid msg_type_flag")),
        }
    }

    pub fn to_frames(self) -> Vec<Frame> {
        match self {
            RequestMessage::Get{user_id, id, path} => {
                let mut frames = vec![];
                if let Some(id) = id {
                    frames.push(Frame::new(Some(user_id), 1, 'G' as u8, Bytes::from(id.clone())));
                } else {
                    if let Some(path) = path {
                        frames.push(Frame::new(Some(user_id), 1, 'P' as u8, Bytes::from(path.clone())));
                    }
                }
                frames
            },
            RequestMessage::Put{user_id, id, parent, data} => {
                put_set_frames(user_id, 'p' as u8, id, parent, false, data)
            },
            RequestMessage::Set{user_id, id, data} => {
                let frames = put_set_frames(user_id, 's' as u8, id, None, true, data);
                // println!("set frames: {:?}\n", frames);
                frames
            },
            RequestMessage::Remove{user_id, id} => {
                // TODO - implementme
                let mut frames = vec![];
                frames.push(Frame::new(Some(user_id), 1, 'R' as u8, Bytes::from(id.clone())));
                frames
            },
        }
    }

    pub fn user_id(&self) -> Option<String> {
        let q = match &self {
            &RequestMessage::Put { user_id, .. } => user_id.clone(),
            &RequestMessage::Set { user_id, .. } => user_id.clone(),
            &RequestMessage::Get { user_id, .. } => user_id.clone(),
            &RequestMessage::Remove { user_id, .. } => user_id.clone(),
        };
        Some(q)
    }
}

fn put_set_frames(user_id: String, msg_typ_code: u8, id: String, parent: Option<String>, is_set: bool, blob_data: Bytes) -> Vec<Frame> {
    let mut frames = vec![];
    let fr_sz = DATA_BYTES_PER_FRAME;
    let mut buf = BytesMut::with_capacity(blob_data.len() + 36 + 36);
    buf.put_slice(id.as_bytes());
    if let Some(pid) = parent {
        buf.put_slice(pid.as_bytes());
    } else {
        if !is_set {
            // only print the null uuid if this is a put
            for _ in 0..36 {
                buf.put_u8(0 as u8);
            }

        }
    }
    buf.put(blob_data);

    let mut data = Bytes::from(buf);

    let mut n_frames = data.len() / fr_sz;
    if data.len() % fr_sz != 0 {
        n_frames += 1;
    }
    let mut bytes_left = data.len();
    let mut ctr = 0;
    let mut uid_opt = Some(user_id.clone());
    while bytes_left > 0 {
        println!("ctr = {}, bytes left = {}", ctr, bytes_left);
        let bytes_to_write = bytes_left.min(fr_sz);
        let fr_dat = data.split_to(bytes_to_write);
        let mtc = if ctr == 0 { msg_typ_code } else { 'd' as u8 }; // Continued data frame
        frames.push(Frame::new(uid_opt, (n_frames - ctr) as u32, mtc, fr_dat));
        uid_opt = None;
        ctr += 1;
        bytes_left -= bytes_to_write;
    }
    frames
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::protocol::wire::try_parse_frame;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_get_id_to_frames() {
        let id_str = String::from("2ab3da63-e24f-47e2-9b56-f3d19fade0cf");
        let msg = RequestMessage::Get {user_id: "2ab3da63-e24f-47e2-9b56-f3d19fade0cf".to_string(), id: Some(id_str.clone()), path: None };
        let frames = msg.to_frames();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].size(), 49+36);
        assert_eq!(String::from_utf8(frames[0].data.to_vec()).unwrap().eq(&id_str), true);
    }

    #[test]
    fn test_set_large_msg() {
        let id_str = String::from("2ab3da63-e24f-47e2-9b56-f3d19fade0cf");
        let mut data_buf = BytesMut::with_capacity(BUF_CAP*2);
        for i in 0..(BUF_CAP*2) {
            data_buf.put_u8(3 as u8);
        }
        let msg = RequestMessage::Set {user_id: "2ab3da63-e24f-47e2-9b56-f3d19fade0cf".to_string(),  id: id_str.clone(), data: data_buf.freeze() };
        let frames = msg.to_frames();

        assert_eq!(frames.len(), 3);

        for i in 0..(frames.len() - 1) {
            if i == 0 {
                assert_eq!(frames[i].size(), 13 + 36 + DATA_BYTES_PER_FRAME);
            } else {
                assert_eq!(frames[i].size(), 13 + DATA_BYTES_PER_FRAME);
            }
        }

        let mut new_buf = BytesMut::with_capacity(BUF_CAP*2);
        for fi in 0..frames.len() {
            let mut f = frames[fi].clone();
            if fi > 0 {
                new_buf.put(f.data);
            } else {
                let _ = f.data.split_to(72);
                new_buf.put(f.data);
            }
        }

        let new_bytes = new_buf.to_vec();
        for b in new_bytes {
            assert_eq!(b, 3 as u8);
        }
    }

    #[test]
    fn test_data_frames() {
        let small_data = "hello";
        let msg = ResponseMessage::Data { data: Bytes::from(small_data) };
        let frames = msg.to_frames();
        assert_eq!(frames.len(), 1);
        for fi in 0..frames.len() {
            let f = &frames[fi];
            let bs = f.to_bytes();
            let mut curs = Cursor::new(&bs[..]);
            let f2 = try_parse_frame(&mut curs, bs.len());
            let frame2 = f2.unwrap();
            assert_eq!(frame2.msg_type_flag == ('d' as u8), true);
            let s = String::from_utf8(frame2.data.to_vec()).unwrap();
            assert!(s.eq("hello"));
        }

        let res_msg = ResponseMessage::from_frames(frames).unwrap();
        println!("resmsg = {:?}", &res_msg);
        let b = match res_msg {
            ResponseMessage::Data { data }=> {
                let dats = String::from_utf8(data.to_vec()).unwrap();
                dats.eq("hello")
            },
            _ => false
        };
        assert!(b);
    }

    #[test]
    fn test_error_frames() {
        let small_data = "hello";
        let msg = ResponseMessage::Error { code: 128, description: String::from(small_data) };
        let frames = msg.to_frames();
        assert_eq!(frames.len(), 1);

        let res_msg = ResponseMessage::from_frames(frames).unwrap();
        println!("resmsg = {:?}", &res_msg);
        let b = match res_msg {
            ResponseMessage::Error { code, description } => {
                (code == 128) && (description.eq("hello"))
            },
            _ => false
        };
        assert!(b);
    }

    #[test]
    fn test_put_frames() {
        let msg = RequestMessage::Put { user_id: "2ab3da63-e24f-47e2-9b56-f3d19fade0cf".to_string(), parent: None, id: "2ab3da63-e24f-47e2-9b56-f3d19fade0ce".to_string(), data: Bytes::from("{\"title\": \"abcdef\"}") };
        let frames = msg.to_frames();
        println!("frame 0: {:?}", &frames[0]);
        let req_msg_res = RequestMessage::from_frames(frames);
        if req_msg_res.is_err() {
            println!("error = {:?}", req_msg_res.unwrap_err());
            panic!("error");
        }
        let req_msg = req_msg_res.unwrap();
        println!("req_msg: {:?}", req_msg);
        let b = match req_msg {
            RequestMessage::Put { user_id, id, data, parent } => {
                let b1 = user_id.eq("2ab3da63-e24f-47e2-9b56-f3d19fade0cf");
                let b2 = id.eq("2ab3da63-e24f-47e2-9b56-f3d19fade0ce");
                let b3 = String::from_utf8(data.to_vec()).unwrap().eq("{\"title\": \"abcdef\"}");
                let b4 = parent.is_none();
                println!("bools {} {} {} {}", b1, b2, b3, b4);
                b1 && b2 && b3 && b4
            },
            _ => false,
        };
        assert!(b);
    }

    #[test]
    fn test_set_frames() {
        let msg = RequestMessage::Set { user_id: "2ab3da63-e24f-47e2-9b56-f3d19fade0cf".to_string(), id: "2ab3da63-e24f-47e2-9b56-f3d19fade0ce".to_string(), data: Bytes::from("{\"title\": \"abcdef\"}") };
        let frames = msg.to_frames();
        println!("frame 0: {:?}", &frames[0]);
        let req_msg_res = RequestMessage::from_frames(frames);
        if req_msg_res.is_err() {
            println!("error = {:?}", req_msg_res.unwrap_err());
            panic!("error");
        }
        let req_msg = req_msg_res.unwrap();
        println!("req_msg: {:?}", req_msg);
        let b = match req_msg {
            RequestMessage::Set { user_id, id, data } => {
                let b1 = user_id.eq("2ab3da63-e24f-47e2-9b56-f3d19fade0cf");
                let b2 = id.eq("2ab3da63-e24f-47e2-9b56-f3d19fade0ce");
                let b3 = String::from_utf8(data.to_vec()).unwrap().eq("{\"title\": \"abcdef\"}");
                println!("bools {} {} {}", b1, b2, b3);
                b1 && b2 && b3
            },
            _ => false,
        };
        assert!(b);
    }

    #[test]
    fn test_data_frames_big() {
        let mut big_data = BytesMut::with_capacity(BUF_CAP*4);
        for _i in 0..(BUF_CAP*4) { 
            big_data.put_u8(3 as u8);
        }
        let msg = ResponseMessage::Data { data: big_data.freeze() };
        let frames = msg.to_frames();
        assert_eq!(frames.len(), 5);

        let res_msg = ResponseMessage::from_frames(frames).unwrap();
        println!("resmsg = {:?}", &res_msg);
        let b = match res_msg {
            ResponseMessage::Data { data } => {
                let bs = data.to_vec();
                bs.len() == (BUF_CAP*4) && bs.iter().map(|x| *x == (3 as u8)).reduce(|x,y| x && y).unwrap_or(false)
            },
            _ => false
        };
        assert!(b);
    }

}
