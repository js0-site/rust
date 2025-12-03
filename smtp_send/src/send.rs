use aok::{OK, Void};
use idoh::{MxLookup, mx::cache::Cache};
use mail_send::{SmtpClientBuilder, smtp::message::Message};
use mail_struct::{DomainMail, Mail};

pub async fn send_mx<'a>(server: &str, mail: Message<'a>) -> Void {
  let mut smtp = SmtpClientBuilder::new(server, 25)
    .implicit_tls(false)
    .connect()
    .await?;

  Ok(smtp.send(mail).await?)
}

pub async fn send(mail: Mail) -> Void {
  for DomainMail { domain, mail } in mail.domain_mail() {
    match Cache.mx(&domain).await {
      Ok(mx_li) => {
        if let Some(mx_li) = mx_li {
          'out: {
            for mx in mx_li.iter() {
              match send_mx(&mx.server, mail.clone()).await {
                Ok(_) => {
                  break 'out;
                }
                Err(e) => {
                  log::error!("❌ {domain} → {}:25 : {e}", mx.server);
                }
              }
            }
            log::error!("❌ {domain} all mx failed");
          }
        } else {
          log::error!("{domain} DNS NO MX");
        }
      }
      Err(e) => {
        log::error!("{domain} MX lookup failed: {e}");
        continue;
      }
    }
  }

  OK
}
