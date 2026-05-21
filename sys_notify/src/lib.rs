#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(target_os = "linux")]
pub fn ready() -> bool {
  sd_notify::notify(&[sd_notify::NotifyState::Ready]).is_ok()
}

#[cfg(not(target_os = "linux"))]
pub fn ready() -> bool {
  // 非 Linux 平台什么都不做，或者打印日志
  true
}

#[cfg(target_os = "linux")]
pub fn mainid(pid: u32) -> bool {
  sd_notify::notify(&[sd_notify::NotifyState::MainPid(pid)]).is_ok()
}

#[cfg(not(target_os = "linux"))]
pub fn mainid(_pid: u32) -> bool {
  true
}

#[cfg(target_os = "linux")]
pub fn stopping() -> bool {
  sd_notify::notify(&[sd_notify::NotifyState::Stopping]).is_ok()
}

#[cfg(not(target_os = "linux"))]
pub fn stopping() -> bool {
  true
}
