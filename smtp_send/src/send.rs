use aok::{OK, Void};
use mail_send::SmtpClientBuilder;
use mail_struct::{DomainMail, Mail};

pub async fn send(mail: Mail) -> Void {
  for DomainMail { domain, mail } in mail.domain_mail() {
    let mut mx_li = match idoh::mx(&domain).await {
      Ok(mx_li) => mx_li,
      Err(e) => {
        log::error!("{domain} MX lookup failed: {e}");
        continue;
      }
    };
    mx_li.sort_by(|a, b| a.priority.cmp(&b.priority));
    'out: {
      for mx in mx_li {
        match SmtpClientBuilder::new(&mx.server, 25)
          .implicit_tls(false)
          .connect()
          .await
        {
          Ok(mut client) => match client.send(mail.clone()).await {
            Ok(_) => {
              break 'out;
            }
            Err(e) => {
              log::error!("❌ {domain} → {}:25 : {e}", mx.server);
            }
          },
          Err(e) => {
            log::error!("❌ {domain} → {}:25 Connection failed: {e}", mx.server);
          }
        }
      }
      log::error!("❌ {domain} all mx failed");
    }
  }

  OK
}
