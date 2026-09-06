#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(not(feature = "off"))]
#[macro_export]
macro_rules! code_turn {
  ($($arg:tt)*) => {
    $($arg)*
  };
}

#[cfg(feature = "off")]
#[macro_export]
macro_rules! code_turn {
  ($($arg:tt)*) => {};
}
