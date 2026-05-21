use std::str::from_utf8;

use aok::{OK, Void};
use fred::interfaces::KeysInterface;
use mail_send::mail_auth::common::crypto::{RsaKey, Sha256};
use mail_struct::{Mail, UserMail};
use smtp_send::{Send, send};

use crate::r::{DOMAIN_HOST, HOST_DKIM, HOST_DKIM_KEY};

#[derive(Clone)]
pub struct Mailer {
  sk: Option<[u8; 32]>,
}

impl Mailer {
  pub fn new(sk: Option<impl AsRef<[u8]>>) -> Self {
    let sk = sk.and_then(|s| {
      let s = s.as_ref();
      if s.len() == 32 {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(s);
        Some(arr)
      } else {
        None
      }
    });
    Self { sk }
  }
}

async fn sign_and_send(sk: &[u8; 32], host_id_bytes: &[u8], mut mail: Mail) -> Void {
  let host_dkim_key = [HOST_DKIM, host_id_bytes].concat();
  let host_dkim_private_key = [HOST_DKIM_KEY, host_id_bytes].concat();

  let selector_bytes: Option<Vec<u8>> = xkv::R.get(&host_dkim_key[..]).await.ok().flatten();
  if let Some(selector_bytes) = selector_bytes
    && let Ok(selector) = from_utf8(&selector_bytes)
  {
    let private_key_der: Option<Vec<u8>> = xkv::R.get(&host_dkim_private_key[..]).await.ok().flatten();
    if let Some(der) = private_key_der {
      let rsa_key = RsaKey::<Sha256>::from_key_der(
        rustls_pki_types::PrivateKeyDer::Pkcs8(
          rustls_pki_types::PrivatePkcs8KeyDer::from(der.clone()),
        )
      ).or_else(|_| {
        RsaKey::<Sha256>::from_key_der(
          rustls_pki_types::PrivateKeyDer::Pkcs1(
            rustls_pki_types::PrivatePkcs1KeyDer::from(der),
          )
        )
      });

      if let Ok(rsa_key) = rsa_key {
        let signer = std::sync::Arc::new(
          mail_send::mail_auth::dkim::DkimSigner::from_key(rsa_key)
            .domain(&mail.sender_host)
            .selector(selector)
            .headers([
              "From", "From", "Subject", "Subject", "Date", "Date", "To", "To",
              "Cc", "Cc", "Message-ID", "List-Unsubscribe", "List-Unsubscribe",
              "List-Unsubscribe-Post", "List-Unsubscribe-Post"
            ]),
        );
        let _ = smtp_send::send(&mail, Some(&signer)).await;
        return OK;
      }
    }

    let smtp = Send::new(selector, sk);
    let _ = smtp.send(&mut mail).await;
    return OK;
  }
  let _ = send(&mail, None).await;
  OK
}

impl smtp_recv::Mailer for Mailer {
  async fn send(&self, UserMail { user_id, mail }: UserMail) -> Void {
    if let Some(sk) = &self.sk {
      sign_and_send(sk, &intbin::to_bin(user_id), mail).await
    } else {
      let _ = send(&mail, None).await;
      OK
    }
  }

  async fn forward(&self, mail: Mail) -> Void {
    let domain_key = [DOMAIN_HOST, mail.sender_host.as_bytes()].concat();
    let host_id_bytes: Option<Vec<u8>> = xkv::R.get(&domain_key[..]).await.ok().flatten();
    if let Some(sk) = &self.sk
      && let Some(id_bytes) = host_id_bytes
    {
      sign_and_send(sk, &id_bytes, mail).await
    } else {
      let _ = send(&mail, None).await;
      OK
    }
  }
}
