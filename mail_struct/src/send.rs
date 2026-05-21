use std::{borrow::Cow, collections::hash_map, vec};

use mail_send::smtp::message::{Address, IntoMessage, Message, Parameters};

use crate::Mail;

type HashMapIter<'a, K, V> = hash_map::Iter<'a, K, V>;

pub struct MailMessage<'a> {
  pub sender_user: &'a str,
  pub sender_host: &'a str,
  pub domain: &'a str,
  pub to_li: Vec<Address<'a>>,
  pub body: &'a [u8],
}

impl<'a> IntoMessage<'a> for MailMessage<'a> {
  fn into_message(self) -> mail_send::Result<Message<'a>> {
    (&self).into_message()
  }
}

impl<'a> IntoMessage<'a> for &MailMessage<'a> {
  fn into_message(self) -> mail_send::Result<Message<'a>> {
    Ok(Message::new(
      Address::new(
        format!("{}@{}", self.sender_user, self.sender_host),
        Parameters::new(),
      ),
      self.to_li.clone(),
      Cow::Borrowed(self.body),
    ))
  }
}

pub struct MailMessageItem<'a> {
  pub sender_user: &'a str,
  pub sender_host: &'a str,
  pub to: Address<'a>,
  pub body: &'a [u8],
}

impl<'a> IntoMessage<'a> for &MailMessageItem<'a> {
  fn into_message(self) -> mail_send::Result<Message<'a>> {
    Ok(Message::new(
      Address::new(
        format!("{}@{}", self.sender_user, self.sender_host),
        Parameters::new(),
      ),
      vec![self.to.clone()],
      Cow::Borrowed(self.body),
    ))
  }
}

pub struct MailMessageIter<'a> {
  sender_user: &'a str,
  sender_host: &'a str,
  body: &'a [u8],
  to_li: vec::IntoIter<Address<'a>>,
}

impl<'a> Iterator for MailMessageIter<'a> {
  type Item = MailMessageItem<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    self.to_li.next().map(|to| MailMessageItem {
      sender_user: self.sender_user,
      sender_host: self.sender_host,
      to,
      body: self.body,
    })
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.to_li.size_hint()
  }
}

impl<'a> ExactSizeIterator for MailMessageIter<'a> {
  fn len(&self) -> usize {
    self.to_li.len()
  }
}

impl<'a> IntoIterator for MailMessage<'a> {
  type Item = MailMessageItem<'a>;
  type IntoIter = MailMessageIter<'a>;

  fn into_iter(self) -> Self::IntoIter {
    MailMessageIter {
      sender_user: self.sender_user,
      sender_host: self.sender_host,
      body: self.body,
      to_li: self.to_li.into_iter(),
    }
  }
}

pub struct MailIter<'a> {
  sender_user: &'a str,
  sender_host: &'a str,
  body: &'a [u8],
  hosts: HashMapIter<'a, String, rapidhash::RapidHashSet<String>>,
}

impl<'a> Iterator for MailIter<'a> {
  type Item = MailMessage<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    self.hosts.next().map(|(host, user_set)| {
      let to_li: Vec<Address<'a>> = user_set
        .iter()
        .map(|user| {
          let email = format!("{}@{}", user, host);
          Address::new(email, Parameters::new())
        })
        .collect();
      MailMessage {
        domain: host,
        sender_user: self.sender_user,
        sender_host: self.sender_host,
        to_li,
        body: self.body,
      }
    })
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.hosts.size_hint()
  }
}

impl<'a> ExactSizeIterator for MailIter<'a> {
  fn len(&self) -> usize {
    self.hosts.len()
  }
}

impl<'a> IntoIterator for &'a Mail {
  type Item = MailMessage<'a>;
  type IntoIter = MailIter<'a>;

  fn into_iter(self) -> Self::IntoIter {
    MailIter {
      sender_user: &self.sender_user,
      sender_host: &self.sender_host,
      body: self.body.as_slice(),
      hosts: self.host_user_li.iter(),
    }
  }
}
