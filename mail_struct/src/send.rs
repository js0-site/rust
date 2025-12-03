use std::{borrow::Cow, collections::HashMap};

use mail_send::smtp::message::{Address, Message, Parameters};

use crate::Mail;

pub struct DomainMail<'a> {
  pub domain: &'a str,
  pub mail: Message<'a>,
}

impl Mail {
  pub fn domain_mail<'a>(&'a self) -> Vec<DomainMail<'a>> {
    let mut domain_to_li = HashMap::<&str, Vec<&str>>::new();
    for to in &self.to_li {
      if let Some(p) = to.as_str().rfind('@') {
        let domain = &to[p + 1..];
        domain_to_li.entry(domain).or_default().push(to);
      }
    }
    domain_to_li
      .into_iter()
      .map(|(domain, to_li)| DomainMail {
        domain,
        mail: Message::new(
          Address::new(&self.sender, Parameters::new()),
          to_li
            .into_iter()
            .map(|t| Address::new(t, Parameters::new())),
          Cow::Borrowed(self.body.as_slice()),
        ),
      })
      .collect()
  }
}
