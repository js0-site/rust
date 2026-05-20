use fred::{interfaces::StreamsInterface, prelude::FredResult, types::Key};
use xkv::R;

pub async fn auto_new<T>(key: &Key, group: &str, result: FredResult<T>) -> FredResult<Option<T>> {
  Ok(match result {
    Ok(t) => Some(t),
    Err(e) => {
      if e.details().starts_with("NOGROUP ") {
        let _: () = R
          .xgroup_create::<(), _, _, _>(key, group, "0", true)
          .await?;
        None
      } else {
        return Err(e);
      }
    }
  })
}
