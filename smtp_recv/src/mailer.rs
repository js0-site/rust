use std::future::Future;

use anyhow::Result;
use mail_struct::UserMail;

/// 邮件发送处理器trait
///
/// 实现此trait以定义如何处理接收到的邮件。
/// 邮件可以被转发、存储到数据库、写入文件系统等。
///
/// # 示例
///
/// ```no_run
/// use smtp_recv::Mailer;
/// use mail_struct::UserMail;
///
/// struct MyMailer;
///
/// impl Mailer for MyMailer {
///     async fn send(&self, user_mail: UserMail) -> anyhow::Result<()> {
///         // 处理邮件，例如：
///         // - 转发到其他邮件服务器
///         // - 存储到数据库
///         // - 写入文件
///         let mail = user_mail.mail;
///         println!("收到邮件: user_id={}, 发件人={}@{}, 收件人={:?}",
///                  user_mail.user_id, mail.sender_user, mail.sender_host, mail.host_user_li);
///         Ok(())
///     }
/// }
/// ```
pub trait Mailer: Send + Sync + 'static {
  /// 处理接收到的邮件
  ///
  /// # 参数
  ///
  /// - `mail`: 包含邮件信封和内容的Mail结构
  ///
  /// # 返回
  ///
  /// 返回处理结果，如果处理失败会记录错误日志
  fn send(&self, mail: UserMail) -> impl Future<Output = Result<()>> + Send;
}
