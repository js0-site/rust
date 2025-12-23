use aok::Result;

#[derive(Clone)]
pub struct Cert;

impl ssl_trait::CertByHost for Cert {
  type Item = cert_by_host::Cert;
  async fn get(&self, host: &str) -> Result<Option<Self::Item>> {
    cert_by_host::CertByHost
      .get(if let Some((_, tld)) = host.split_once(".") {
        tld
      } else {
        host
      })
      .await
  }
}
