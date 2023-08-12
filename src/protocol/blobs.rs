use bytes::Bytes;
use serde_json::Value;
use anyhow::*;

pub fn extract_owner_id(mut raw: Bytes) -> Result<String> {
    let bs = raw.split_to(raw.len());
    let str_v = String::from_utf8(bs.to_vec()).ok();
    if let Some(x) = str_v {
        let json_val: Value = serde_json::from_str(&x[..])?;
        let uid_v = &json_val["owner_id"];
        match uid_v {
            Value::String(s) => Ok(s.to_string()),
            _ => Err(anyhow!("invalid owner_id")),
        }

    } else {
        Err(anyhow!("no owner_id"))
    }
}

pub fn extract_title(mut raw: Bytes) -> Result<String> {
    let bs = raw.split_to(raw.len());
    let str_v = String::from_utf8(bs.to_vec()).ok();
    if let Some(x) = str_v {
        let json_val: Value = serde_json::from_str(&x[..])?;
        let uid_v = &json_val["title"];
        match uid_v {
            Value::String(s) => Ok(s.to_string()),
            _ => Err(anyhow!("invalid title")),
        }

    } else {
        Err(anyhow!("invalid blob"))
    }
}
