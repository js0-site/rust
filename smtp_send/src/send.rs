use idoh::{MxLookup, mx::cache::Cache};
use log::info;
use mail_struct::MailMessage;

use crate::{Error, SendResult, Signer, Smtp};

pub async fn send<'a>(msg: MailMessage<'a>, signer: &Signer) -> SendResult {
  let mut success = 0;
  let mut error_li = Vec::new();

  let domain = msg.domain;
  match Cache.mx(domain).await {
    Ok(mx_li) => {
      if let Some(mx_li) = mx_li
        && !mx_li.is_empty()
      {
        let len = msg.to_li.len();
        match len {
          0 => {
            return SendResult { success, error_li };
          }
          1 => {
            let email = &msg.to_li[0].email;
            for i in mx_li.iter() {
              match Smtp::connect(&i.server, signer).await {
                Ok(mut smtp) => match smtp.send(&msg).await {
                  Ok(_) => {
                    success += 1;
                    info!("✅ {}", email);
                    break;
                  }
                  Err(err) => {
                    if let mail_send::Error::UnexpectedReply(err) = err {
                      error_li.push(Error::Reject(email.to_string(), err));
                      break;
                    }
                    error_li.push(Error::SendErr(email.to_string(), err));
                  }
                },
                Err(err) => {
                  error_li.push(Error::SendErr(email.to_string(), err));
                }
              }
            }
          }
          _ => 'out: {
            let mut smtp_li = Vec::new();
            let mut send_failed = false;
            let mut error = None;
            for mx in mx_li.iter() {
              match Smtp::connect(&mx.server, signer).await {
                Ok(mut smtp) => {
                  if !send_failed {
                    if smtp.send(&msg).await.is_ok() {
                      success += len;
                      break 'out;
                    } else {
                      send_failed = true;
                    }
                  }
                  smtp_li.push(smtp);
                }
                Err(err) => {
                  error = Some(err);
                }
              }
            }
            if smtp_li.is_empty() {
              if let Some(err) = error {
                error_li.push(Error::SmtpAllFailed(domain.into(), err));
              }
              break 'out;
            }
            'msg: for i in msg {
              error = None;
              let email = &i.to.email;
              for smtp in smtp_li.iter_mut() {
                if let Err(err) = smtp.client.rset().await {
                  error = Some(err);
                  continue;
                }
                match smtp.send(&i).await {
                  Ok(_) => {
                    success += 1;
                    info!("✅ {email}");
                    continue 'msg;
                  }
                  Err(err) => {
                    if let mail_send::Error::UnexpectedReply(err) = err {
                      error_li.push(Error::Reject(email.to_string(), err));
                      continue 'msg;
                    }
                    error = Some(err);
                  }
                }
              }
              if let Some(err) = error {
                error_li.push(Error::SendErr(email.to_string(), err));
              }
            }
          }
        }
      } else {
        error_li.push(Error::MxIsEmpty(domain.into()));
      }
    }
    Err(e) => {
      error_li.push(Error::DnsResolveFailed(domain.into(), e));
    }
  }
  SendResult { success, error_li }
}
