use std::{
  fs::{File, create_dir_all},
  path::PathBuf,
  sync::Arc,
  time::Duration,
};

use fred::{
  interfaces::KeysInterface,
  types::{Expiration, SetOptions},
};
use fs4::fs_std::FileExt;
use tokio::sync::Notify;
use xkv::R;

use crate::{Error, Layout, Result};

/// Redis key prefix for process ID allocation
/// Redis 键前缀，用于进程号分配
const PREFIX: &[u8] = b"sfid:";

/// Lock file directory
/// 锁文件目录
const LOCK_DIR: &str = "/tmp/sfid";

/// Heartbeat interval (3 minutes)
/// 心跳间隔，3分钟
const HEARTBEAT: Duration = Duration::from_secs(3 * 60);

/// Expiration time (10 minutes)
/// 过期时间，10分钟
const EXPIRE: i64 = 10 * 60;

/// Process ID with heartbeat, stops on drop
/// 带心跳的进程号，drop 时自动停止
pub struct Pid {
  id: u16,
  cancel: Arc<Notify>,
  _lock: File,
}

impl Pid {
  #[inline]
  pub fn id(&self) -> u16 {
    self.id
  }
}

impl Drop for Pid {
  fn drop(&mut self) {
    self.cancel.notify_one();
  }
}

/// Get local identity: machine_uid + local_seq
/// 获取本地标识：机器ID + 本地序号
fn local_identity(app: &[u8], max_pid: u32) -> Result<(Box<[u8]>, u16, File)> {
  let machine_id = machine_uid::get().map_err(|e| Error::MachineId(e.to_string()))?;

  let dir: PathBuf = [LOCK_DIR, &String::from_utf8_lossy(app)].iter().collect();
  create_dir_all(&dir).map_err(Error::LockFile)?;

  for seq in 0..max_pid {
    let path = dir.join(seq.to_string());
    let file = match File::create(&path) {
      Ok(f) => f,
      Err(e) => {
        log::warn!("create lock file {path:?}: {e}");
        continue;
      }
    };

    if file.try_lock_exclusive().is_ok() {
      let identity = xbin::concat!(machine_id.as_bytes(), b":", &(seq as u16).to_le_bytes());
      return Ok((identity.into(), seq as u16, file));
    }
  }

  Err(Error::NoAvailablePid(max_pid))
}

/// Extract pid from key (last 2 bytes)
/// 从 key 中提取 pid（最后2字节）
fn pid_from_key(key: &[u8]) -> u16 {
  let len = key.len();
  debug_assert!(len >= 2);
  u16::from_le_bytes([key[len - 2], key[len - 1]])
}

/// Allocate a process ID from Redis
/// 从 Redis 分配进程号
pub async fn allocate<L: Layout>(app: impl AsRef<[u8]>) -> Result<Pid> {
  let app = app.as_ref();
  let max_pid = L::MAX_PID;
  let (local, local_seq, lock_file) = local_identity(app, max_pid)?;

  let prefix = xbin::concat!(PREFIX, app, b":");
  let start = local_seq as u32;
  let expire = Expiration::EX(EXPIRE);

  for i in 0..max_pid {
    let id = ((start + i) % max_pid) as u16;
    let key = xbin::concat!(&*prefix, &id.to_le_bytes());

    let old: Option<Vec<u8>> = R
      .set(
        &*key,
        &*local,
        Some(expire.clone()),
        Some(SetOptions::NX),
        true,
      )
      .await?;

    match old {
      None => {
        if i > 16 {
          let app = String::from_utf8_lossy(app);
          log::info!("[{app}] pid allocated after {i} attempts");
        }
        return Ok(start_heartbeat(key.into(), local, expire, lock_file));
      }
      Some(v) if *v == *local => return Ok(start_heartbeat(key.into(), local, expire, lock_file)),
      _ => {}
    }
  }

  Err(Error::NoAvailablePid(max_pid))
}

/// Start heartbeat and return Pid
/// 启动心跳并返回 Pid
fn start_heartbeat(key: Box<[u8]>, local: Box<[u8]>, expire: Expiration, lock_file: File) -> Pid {
  let id = pid_from_key(&key);
  let cancel = Arc::new(Notify::new());
  let notify = cancel.clone();

  tokio::spawn(async move {
    loop {
      tokio::select! {
        _ = notify.notified() => break,
        _ = tokio::time::sleep(HEARTBEAT) => {
          if let Err(e) = R
            .set::<(), _, _>(&*key, &*local, Some(expire.clone()), None, false)
            .await
          {
            log::error!("heartbeat set: {e}");
          }
        }
      }
    }
  });

  Pid { id, cancel, _lock: lock_file }
}
