pub use kind2sql::Kind;

#[derive(Debug)]
pub struct Meta {
  pub name: &'static str,
  pub kind_li: &'static [&'static str],
  pub field_li: &'static [Kind],
}

impl Meta {
  pub fn dump(&self) -> Vec<u8> {
    let mut r = Vec::new();
    r.push(self.name.len() as u8);
    r.extend_from_slice(self.name.as_bytes());
    r.push(self.kind_li.len() as u8);
    for i in self.kind_li {
      r.extend_from_slice(i.as_bytes());
      r.push(0);
    }
    for i in self.field_li {
      r.push(*i as u8);
    }
    r
  }
}
