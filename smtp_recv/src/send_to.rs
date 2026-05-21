use std::mem;

/// 邮件发送目标信息，包含发件人和收件人列表
///
/// 这个结构体被SMTP会话用来管理邮件的发送方和接收方信息
#[derive(Debug, Clone, Default)]
pub struct SendTo {
  /// 收件人列表
  pub to_li: Vec<String>,
  /// 发件人
  pub sender: Option<String>,
}

impl SendTo {
  /// 创建新的空SendTo实例
  pub fn new() -> Self {
    Self::default()
  }

  /// 重置发送目标状态，清空收件人列表和发件人
  pub fn reset(&mut self) {
    self.to_li.clear();
    self.sender = None;
  }

  /// 设置发件人
  pub fn set_sender(&mut self, sender: String) {
    self.sender = Some(sender);
  }

  /// 添加收件人
  pub fn add_to(&mut self, recipient: String) {
    self.to_li.push(recipient);
  }

  /// 检查是否有发件人
  pub fn has_sender(&self) -> bool {
    self.sender.is_some()
  }

  /// 检查收件人列表是否为空
  pub fn is_empty(&self) -> bool {
    self.to_li.is_empty()
  }

  /// 取出发件人，留下None
  pub fn take_sender(&mut self) -> Option<String> {
    self.sender.take()
  }

  /// 取出收件人列表，留下空Vec
  pub fn take_to_li(&mut self) -> Vec<String> {
    mem::take(&mut self.to_li)
  }

  /// 检查是否准备好发送邮件（有发件人且有收件人）
  pub fn is_ready(&self) -> bool {
    self.has_sender() && !self.is_empty()
  }
}
