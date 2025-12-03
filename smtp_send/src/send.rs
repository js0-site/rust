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

pub async fn send(mail: Mail, retry: u64) -> Void {
  let mut failed = Vec::new();
  'out: for DomainMail { domain, mail } in mail.domain_mail() {
    match Cache.mx(&domain).await {
      Ok(mx_li) => {
        if let Some(mx_li) = mx_li {
          let mut err_li = Vec::new();
          for mx in mx_li.iter() {
            match send_mx(&mx.server, mail.clone()).await {
              Ok(_) => {
                continue 'out;
              }
              Err(e) => {
                let e = e.to_string();
                log::error!("❌ {domain} → {}:25 : {e}", mx.server);
                err_li.push((mx.server.clone(), e));
              }
            }
          }
          log::error!("❌ {domain} all MX send failed");
        } else {
          log::error!("{domain} no MX dns");
        }
      }
      Err(e) => {
        log::error!("{domain} MX lookup failed: {e}");
      }
    }
    for i in mail.rcpt_to {
      failed.push(i.email);
    }
  }

  OK
}
