use std::env;

pub trait Lang {
  fn t(&self, zh: &'static str, en: &'static str) -> &'static str;
}

pub struct ZhLang;
impl Lang for ZhLang {
  fn t(&self, zh: &'static str, _: &'static str) -> &'static str {
    zh
  }
}

pub struct EnLang;
impl Lang for EnLang {
  fn t(&self, _: &'static str, en: &'static str) -> &'static str {
    en
  }
}

pub fn detect_lang() -> Box<dyn Lang> {
  let v = env::var("LANG").unwrap_or_default();
  if v.to_ascii_lowercase().starts_with("zh") {
    Box::new(ZhLang)
  } else {
    Box::new(EnLang)
  }
}
