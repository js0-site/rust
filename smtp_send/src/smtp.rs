use mail_send::{Result, SmtpClient, SmtpClientBuilder, smtp::message::IntoMessage};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

use crate::Signer;

pub struct Smtp<'a> {
  pub client: SmtpClient<TlsStream<TcpStream>>,
  signer: &'a Signer,
}

impl<'a> Smtp<'a> {
  pub async fn connect(server: &str, signer: &'a Signer) -> Result<Self> {
    Ok(Self {
      signer,
      client: SmtpClientBuilder::new(server, 25)
        .implicit_tls(false)
        .connect()
        .await?,
    })
  }

  pub async fn send(&mut self, mail: impl IntoMessage<'_>) -> Result<()> {
    self.client.send_signed(mail, self.signer).await
  }
}
