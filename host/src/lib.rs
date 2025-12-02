#![cfg_attr(docsrs, feature(doc_cfg))]

use global_str::GlobalStr;
use static_init::constructor;

pub static HOST: GlobalStr = GlobalStr::new();

#[constructor(0)]
extern "C" fn init() {
  let s = hostname::get()
    .map(|v| v.to_string_lossy().into_owned())
    .unwrap();
  unsafe {
    *HOST.0.get() = Box::leak(s.into_boxed_str());
  }
}
