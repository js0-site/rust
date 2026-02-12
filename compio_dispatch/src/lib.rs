#![cfg_attr(docsrs, feature(doc_cfg))]

use compio_dispatcher::Dispatcher;
use static_init::dynamic;

#[dynamic]
pub static DISPATCH: Dispatcher = Dispatcher::new().unwrap();
