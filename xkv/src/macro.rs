#[macro_export]
macro_rules! conn {
  ($var:ident) => {
    $crate::conn_with_dollar!($var, $);
  };
}

#[macro_export]
macro_rules! conn_with_dollar {
  ($var:ident, $d:tt) => {
    use $crate::fred::prelude::Client;
    #[macro_export]
    macro_rules! $var {
      ($func:ident $d($d args:expr),*) => {
        let _:() = $crate::$var.$func($d($d args),*).await?;
      };
    }

    static_::init!($var: Client {
      use $crate::{conn, log::{warn, info}};

      let mut retry = 0;
      let prefix = stringify!($var);
      loop {
        match conn(prefix).await {
          Ok(r) => {
            if retry > 0 {
              info!("✅ connected redis {prefix}");
            }
            return Ok(r);
          }
          Err(err) => {
            warn!("❌ redis {prefix} ( retry {} ): {}", retry, err);
            if retry > 99 {
              return Err(err);
            }
            retry += 1;
          }
        }
      }
    });
  };
}
