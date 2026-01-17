use rapidhash::{HashMapExt, HashSetExt, RapidHashMap as HashMap, RapidHashSet as HashSet};

use crate::{Error, Mail, SendResult};

pub fn encode(mail: &Mail, result: &SendResult) -> Vec<u8> {
  if result.success == 0 {
    // 全部失败，返回完整邮件 / All failed, return full mail
    return bitcode::encode(mail);
  }

  // 部分成功，只保留失败的 / Partial success, keep only failed
  let mut m = mail.clone();
  let mut err_host = HashSet::new();
  let mut merge: HashMap<String, HashSet<String>> = HashMap::new();

  for e in &result.error_li {
    match e {
      Error::DnsResolveFailed(h, _) | Error::SmtpAllFailed(h, _) | Error::MxIsEmpty(h) => {
        err_host.insert(h);
      }
      Error::Reject(i) => insert(&mut merge, &i.email),
      Error::SendErr(i) => insert(&mut merge, &i.email),
      // DkimInit/TooManyReceived 不重试 / No retry
      Error::DkimInit(_) | Error::TooManyReceived(_) => {}
    }
  }

  // 移除成功的主机 / Remove successful hosts
  m.host_user_li
    .retain(|h, _| err_host.contains(h) || merge.contains_key(h));

  // 合并失败的邮箱 / Merge failed emails
  for (host, users) in merge {
    m.host_user_li.entry(host).or_default().extend(users);
  }

  bitcode::encode(&m)
}

fn insert(merge: &mut HashMap<String, HashSet<String>>, email: &str) {
  if let Some((user, host)) = email.split_once('@') {
    merge.entry(host.into()).or_default().insert(user.into());
  }
}
