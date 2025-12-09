pub mod create_tar_zstd;
pub mod encode_mail;
pub mod reject_mail;

use idoh::{MxLookup, mx::cache::Cache};
use log::error;
use reject_mail::mail_message;

use crate::{Mail, SendResult, Signer, Smtp};

pub async fn reject(mail: &Mail, result: &SendResult, signer: &Signer) {
  let reject_message = mail_message(mail, result);
  let address = format!("{}@{}", mail.sender_user, mail.sender_host);

  match Cache.mx(&mail.sender_host).await {
    Ok(mx_li) => {
      if let Some(mx_li) = mx_li {
        for mx in mx_li.iter() {
          match Smtp::connect(&mx.server, signer).await {
            Ok(mut smtp) => match smtp.send(&reject_message).await {
              Ok(()) => {
                break;
              }
              Err(e) => {
                error!("reject {address}: {e}");
              }
            },
            Err(e) => {
              error!("reject {address} connect to {}: {e}", mx.server);
            }
          }
        }
      } else {
        error!("reject {address}: mx is empty");
      }
    }
    Err(e) => {
      error!("reject {address} : {}", e);
    }
  }
}
