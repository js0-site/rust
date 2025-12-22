use aok::{OK, Result};
use log::info;
use tokio::time::{Duration, sleep};

pub struct Client {}

impl Client {
  pub async fn test(&self) {
    info!("client test success");
  }
}

pub async fn connect() -> Result<Client> {
  info!("Sleeping for 3 seconds...");
  sleep(Duration::from_secs(3)).await;
  Ok(Client {})
}

xboot::init!(CLIENT: Client {
  connect().await
});

#[tokio::main]
async fn main() -> Result<()> {
  log_init::init();
  xboot::init().await?;
  info!("inited");
  CLIENT.test().await;
  OK
}
