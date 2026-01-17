use auth_trait::Auth;

#[derive(Clone)]
pub struct SimpleAuth;

impl Auth for SimpleAuth {
  async fn verify(
    &self,
    host: &str,
    username: &str,
    password: &str,
  ) -> anyhow::Result<Option<u64>> {
    println!(
      "认证验证: host={} user={} pass={}",
      host, username, password
    );

    // 简单的测试认证：接受任何用户名和密码
    if !username.is_empty() && !password.is_empty() {
      Ok(Some(1)) // 返回用户ID 1
    } else {
      Ok(None) // 认证失败
    }
  }
}
