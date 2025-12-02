use aok::{OK, Void};
use log::info;
use std::collections::HashSet;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test_domain_mail() -> Void {
  use mail_struct::Mail;

  const SENDER: &str = "sender@example.com";
  const GMAIL1: &str = "user1@gmail.com";
  const YAHOO: &str = "user2@yahoo.com";
  const GMAIL2: &str = "user3@gmail.com";
  const HOTMAIL: &str = "user4@hotmail.com";

  let mail = Mail {
    sender: SENDER.to_string(),
    to_li: vec![
      GMAIL1.to_string(),
      YAHOO.to_string(),
      GMAIL2.to_string(),
      HOTMAIL.to_string(),
    ],
    body: b"test body".to_vec(),
  };

  let domain_mail = mail.domain_mail();

  // Should have 3 groups: gmail.com, yahoo.com, hotmail.com
  assert_eq!(domain_mail.len(), 3);

  let mut domains = HashSet::new();

  for item in &domain_mail {
    domains.insert(item.domain);

    let rcpt_to: Vec<&str> = item.mail.rcpt_to.iter().map(|addr| addr.email.as_ref()).collect();
    info!("{}\t{:?}",item.domain, rcpt_to);

    match item.domain {
      "gmail.com" => {
        assert_eq!(rcpt_to, vec![GMAIL1, GMAIL2]);
      }
      "yahoo.com" => {
        assert_eq!(rcpt_to, vec![YAHOO]);
      }
      "hotmail.com" => {
        assert_eq!(rcpt_to, vec![HOTMAIL]);
      }
      _ => panic!("Unexpected domain: {}", item.domain),
    }
  }

  info!("> test domain_mail passed");
  OK
}
