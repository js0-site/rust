use std::ops::{Deref, DerefMut};

#[cfg(feature = "decode")]
use bitcode::Decode;
#[cfg(feature = "encode")]
use bitcode::Encode;
use rapidhash::{RapidHashMap as HashMap, RapidHashSet as HashSet};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "decode", derive(Decode))]
#[cfg_attr(feature = "encode", derive(Encode))]
pub struct HostUserLi(HashMap<String, HashSet<String>>);

impl Deref for HostUserLi {
  type Target = HashMap<String, HashSet<String>>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for HostUserLi {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl HostUserLi {
  // Add email address with validation / 添加邮箱地址并验证
  pub fn add(&mut self, mail: impl AsRef<str>) -> bool {
    let mail = mail.as_ref();
    if let Some((user, host)) = xmail::norm_user_host(mail) {
      self.entry(host).or_default().insert(user);
      true
    } else {
      log::info!("mail invalid: {mail}");
      false
    }
  }

  // Get user_li for host / 获取主机的用户列表
  pub fn user_li(&self, host: &str) -> Option<&HashSet<String>> {
    self.0.get(host)
  }
}

impl<T: AsRef<str>> FromIterator<T> for HostUserLi {
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let mut host_user_li = Self::default();
    for mail in iter {
      host_user_li.add(mail);
    }
    host_user_li
  }
}
