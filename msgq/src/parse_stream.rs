use fred::prelude::Value;

use crate::StreamItem;

pub fn parse_stream(val: Value) -> Vec<StreamItem> {
  let val = val.into_array();
  let mut res = Vec::new();
  for item in val {
    if let Value::Array(li) = item {
      for i in li {
        if let Value::Array(mut li) = i
          && let Some(Value::Integer(retry)) = li.pop()
        {
          let retry = retry as u64;
          if let Some(Value::Integer(idle_ms)) = li.pop() {
            let idle_ms = idle_ms as u64;
            let mut len = li.len() - 1;
            let mut kv = Vec::with_capacity(len);
            while len > 0 {
              len -= 1;
              if let Some(Value::Array(mut key_val)) = li.pop()
                && let Some(Value::Bytes(val)) = key_val.pop()
                && let Some(Value::String(key)) = key_val.pop()
              {
                let key = key.into_inner();
                kv.push((key, val));
              }
            }
            if let Some(Value::String(stream_id)) = li.pop() {
              res.push(StreamItem {
                id: stream_id.to_string(),
                retry,
                kv,
                idle_ms,
              });
            }
          }
        }
      }
    }
  }
  res
}
