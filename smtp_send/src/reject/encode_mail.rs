use std::collections::{HashMap, HashSet};

use bitcode::encode;

use crate::{Error, Mail, SendResult};

pub fn encode_mail(mail: &Mail, result: &SendResult) -> Vec<u8> {
  if result.success == 0 {
    // 全部发送失败，返回完整邮件
    encode(mail)
  } else {
    // 部分发送成功，只保留发送失败的邮箱地址
    let mut mail_clone = mail.clone();
    let host_user_li = &mut mail_clone.host_user_li;
    let mut error_host = HashSet::new();
    let mut to_merge: HashMap<String, HashSet<String>> = HashMap::new();

    // 收集所有发送失败的主机和邮箱地址
    for i in &result.error_li {
      match i {
        Error::DnsResolveFailed(host, _)
        | Error::SmtpAllFailed(host, _)
        | Error::MxIsEmpty(host) => {
          error_host.insert(host);
        }
        Error::Reject(mail, _) | Error::SendErr(mail, _) => {
          if let Some((user, host)) = mail.split_once('@') {
            to_merge.entry(host.into()).or_default().insert(user.into());
          }
        }
      }
    }

    // 移除成功发送的主机
    for i in mail.host_user_li.keys() {
      if !error_host.contains(i) {
        host_user_li.remove(i);
      }
    }

    // 合并部分失败的邮箱地址
    for (host, user_set) in to_merge {
      if let Some(pre) = host_user_li.get_mut(&host) {
        pre.extend(user_set);
      } else {
        host_user_li.insert(host, user_set);
      }
    }
    encode(&mail_clone)
  }
}
