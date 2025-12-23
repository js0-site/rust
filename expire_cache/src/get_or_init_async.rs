/// Async get or init macro / 异步获取或初始化宏
#[macro_export]
macro_rules! get_or_init_async {
  ($cache:expr, $key:expr, $init:expr) => {{
    let key = $key;
    match $cache.get(key) {
      Some(v) => Ok(v),
      None => {
        let val = $init().await?;
        $cache.insert(key.to_owned(), val.clone());
        Ok(val)
      }
    }
  }};
}
