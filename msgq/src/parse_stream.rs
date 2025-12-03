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

            // Parse kv pairs - expecting Array([field, value, field, value, ...])
            let kv = if let Some(Value::Array(kv_array)) = li.pop() {
              let mut result = Vec::new();
              let mut iter = kv_array.into_iter();
              while let Some(key) = iter.next() {
                if let Some(val) = iter.next() {
                  // Convert key to Bytes
                  let key_bytes = match key {
                    Value::String(s) => s.into_inner(),
                    Value::Bytes(b) => b,
                    _ => continue,
                  };
                  // Convert value to Bytes
                  let val_bytes = match val {
                    Value::String(s) => s.into_inner(),
                    Value::Bytes(b) => b,
                    _ => continue,
                  };
                  result.push((key_bytes, val_bytes));
                }
              }
              result
            } else {
              Vec::new()
            };

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
