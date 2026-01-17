use idns::{Mx, Query};
use idot::DOT;
use log::info;
use mail_send::Error::UnexpectedReply;
use mail_struct::MailMessage;

use crate::{Error, Reject, SendErr, SendResult, Signer, Smtp};

pub async fn send(msg: MailMessage<'_>, signer: Option<&Signer>) -> SendResult {
  let mut success = 0;
  let mut error_li = Vec::new();
  let domain = msg.domain;

  let mx_li = match Query::query::<Mx>(&*DOT, domain).await {
    Ok(Some(li)) if !li.is_empty() => li,
    Ok(_) => {
      return SendResult {
        success,
        error_li: vec![Error::MxIsEmpty(domain.into())],
      };
    }
    Err(e) => {
      return SendResult {
        success,
        error_li: vec![Error::DnsResolveFailed(domain.into(), e)],
      };
    }
  };

  let len = msg.to_li.len();
  if len == 0 {
    return SendResult { success, error_li };
  }

  if len == 1 {
    let email = &msg.to_li[0].email;
    for mx in &mx_li {
      match Smtp::connect(&mx.server, signer).await {
        Ok(mut smtp) => match smtp.send(&msg).await {
          Ok(_) => {
            success += 1;
            info!("✅ {} {email}", mx.server);
            break;
          }
          Err(UnexpectedReply(err)) => {
            error_li.push(Error::Reject(Reject {
              email: email.to_string(),
              server: mx.server.clone(),
              response: err,
            }));
            break;
          }
          Err(err) => {
            error_li.push(Error::SendErr(SendErr {
              email: email.to_string(),
              server: mx.server.clone(),
              error: err,
            }));
          }
        },
        Err(err) => {
          error_li.push(Error::SendErr(SendErr {
            email: email.to_string(),
            server: mx.server.clone(),
            error: err,
          }));
        }
      }
    }
    return SendResult { success, error_li };
  }

  // 多收件人：尝试批量发送，失败则逐个发送
  let mut smtp_li = Vec::new();
  let mut last_err = None;

  for mx in &mx_li {
    match Smtp::connect(&mx.server, signer).await {
      Ok(mut smtp) => {
        if smtp_li.is_empty() && smtp.send(&msg).await.is_ok() {
          return SendResult {
            success: len,
            error_li,
          };
        }
        smtp_li.push(smtp);
      }
      Err(err) => last_err = Some(err),
    }
  }

  if smtp_li.is_empty() {
    if let Some(err) = last_err {
      error_li.push(Error::SmtpAllFailed(domain.into(), err));
    }
    return SendResult { success, error_li };
  }

  // 逐个发送
  'msg: for m in msg {
    let email = &m.to.email;
    for (idx, smtp) in smtp_li.iter_mut().enumerate() {
      let server = &mx_li[idx].server;
      if smtp.rset().await.is_err() {
        continue;
      }
      match smtp.send(&m).await {
        Ok(_) => {
          success += 1;
          info!("✅ {server} {email}");
          continue 'msg;
        }
        Err(UnexpectedReply(err)) => {
          error_li.push(Error::Reject(Reject {
            email: email.to_string(),
            server: server.clone(),
            response: err,
          }));
          continue 'msg;
        }
        Err(err) => last_err = Some(err),
      }
    }
    if let Some(err) = last_err.take() {
      let server = mx_li.last().map_or_else(String::new, |m| m.server.clone());
      error_li.push(Error::SendErr(SendErr {
        email: email.to_string(),
        server,
        error: err,
      }));
    }
  }

  SendResult { success, error_li }
}
