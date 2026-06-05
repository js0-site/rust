use std::{io, mem::ManuallyDrop, time::Duration};

use mail_send::{Result, SmtpClient, SmtpClientBuilder, smtp::message::IntoMessage};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

use crate::Signer;

pub struct Smtp<'a> {
  client: ManuallyDrop<SmtpClient<TlsStream<TcpStream>>>,
  signer: Option<&'a Signer>,
}

impl<'a> Smtp<'a> {
  pub async fn connect(server: &str, signer: Option<&'a Signer>) -> Result<Self> {
    Ok(Self {
      signer,
      client: ManuallyDrop::new(
        SmtpClientBuilder::new(server, 25)
          .map_err(io::Error::other)?
          .implicit_tls(false)
          .allow_invalid_certs()
          .timeout(Duration::from_secs(10))
          .connect()
          .await?,
      ),
    })
  }

  pub async fn send(&mut self, mail: impl IntoMessage<'_>) -> Result<()> {
    match self.signer {
      Some(signer) => self.client.send_signed(mail, signer).await,
      None => self.client.send(mail).await,
    }
  }

  // 重置连接状态 / Reset connection state
  pub async fn rset(&mut self) -> Result<()> {
    self.client.rset().await
  }
}

impl Drop for Smtp<'_> {
  fn drop(&mut self) {
    // SAFETY: drop 只会被调用一次 / drop is only called once
    let client = unsafe { ManuallyDrop::take(&mut self.client) };
    tokio::spawn(async move {
      let _ = client.quit().await;
    });
  }
}
