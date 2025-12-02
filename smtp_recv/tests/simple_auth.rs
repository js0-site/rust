use aok::Result;
use auth_trait::Auth;

pub struct SimpleAuth;

impl Auth for SimpleAuth {
  async fn verify(&self, host: &str, username: &str, password: &str) -> Result<Option<u64>> {
    println!("认证验证: host={host} user={username} pass={password}");
    Ok(Some(1))
  }
}
