use std::fmt::Write;

use log::kv;
use logforth::{
  Error as LogforthError,
  kv::{Key as LogforthKey, Visitor, value_bag::ValueBag},
};

pub struct Kv {
  pub text: String,
}

impl<'kvs> kv::VisitSource<'kvs> for Kv {
  fn visit_pair(&mut self, key: kv::Key<'kvs>, value: kv::Value<'kvs>) -> Result<(), kv::Error> {
    write!(&mut self.text, " {key}={value}").unwrap();
    Ok(())
  }
}

impl Visitor for Kv {
  fn visit(&mut self, key: LogforthKey<'_>, value: ValueBag<'_>) -> Result<(), LogforthError> {
    write!(&mut self.text, " {key}={value}").unwrap();
    Ok(())
  }
}
