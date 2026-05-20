use aok::{OK, Void};
use log::info;
use rapidhash::{HashSetExt, RapidHashSet as HashSet};

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

  let mail = Mail::new(SENDER, vec![GMAIL1, YAHOO, GMAIL2, HOTMAIL], b"test body").unwrap();

  // Should have 3 groups: gmail.com, yahoo.com, hotmail.com
  assert_eq!(mail.host_user_li.len(), 3);

  let mut domains = HashSet::new();

  for item in &mail {
    domains.insert(item.domain);

    let rcpt_to: Vec<&str> = item.to_li.iter().map(|addr| addr.email.as_ref()).collect();

    info!("{}	:	{}", item.domain, rcpt_to.join(" / "));

    match item.domain {
      "gmail.com" => {
        assert_eq!(rcpt_to.len(), 2);
        assert!(rcpt_to.contains(&GMAIL1));
        assert!(rcpt_to.contains(&GMAIL2));
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
