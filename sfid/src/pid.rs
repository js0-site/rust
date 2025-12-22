use std::{sync::Arc, time::Duration};

use fred::{
  interfaces::KeysInterface,
  types::{Expiration, SetOptions},
};
use tokio::sync::Notify;
use xkv::R;

use crate::{Error, Result, bits::MAX_PID};

/// Redis key prefix for process ID allocation
/// Redis 键前缀，用于进程号分配
const PREFIX: &[u8] = b"sfid:";

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

/// Extract pid from key (last 2 bytes)
/// 从 key 中提取 pid（最后2字节）
fn pid_from_key(key: &[u8]) -> u16 {
  // Key format: PREFIX + app + ":" + pid_le_bytes (2 bytes)
  // Safe: key always ends with 2-byte pid
  // 键格式：PREFIX + app + ":" + pid_le_bytes（2字节）
  // 安全：key 总是以 2 字节 pid 结尾
  let len = key.len();
  debug_assert!(len >= 2);
  u16::from_le_bytes([key[len - 2], key[len - 1]])
}

/// Allocate a process ID from Redis
/// 从 Redis 分配进程号
pub async fn allocate(app: impl AsRef<[u8]>) -> Result<Pid> {
  let app = app.as_ref();
  let local = uuid::Uuid::new_v4().into_bytes();
  let prefix = xbin::concat!(PREFIX, app, b":");
  let start = rand::random_range(0..MAX_PID);
  let expire = Expiration::EX(EXPIRE);
  let mut last_occupied = 0u16;

  for i in 0..MAX_PID {
    let id = ((start + i) % MAX_PID) as u16;
    let key = xbin::concat!(&*prefix, &id.to_le_bytes());

    // SET key value EX seconds NX GET: returns old value if exists
    // SET key value EX seconds NX GET：若存在则返回旧值
    let old: Option<Vec<u8>> = R
      .set(
        &*key,
        &local[..],
        Some(expire.clone()),
        Some(SetOptions::NX),
        true, // GET
      )
      .await?;

    match old {
      // Key not exist, set success
      // 键不存在，设置成功
      None => {
        if i > 16 {
          let app = String::from_utf8_lossy(app);
          // pid allocated after N attempts, app=X, last occupied=Y
          // 经 N 次尝试后分配到 pid，app=X，上次被占用=Y
          log::info!("[{app}] attempts {i} spid allocated failed , last occupied={last_occupied}");
        }
        return Ok(start_heartbeat(key.into(), local, expire));
      }
      // Already owned by us
      // 已被自己持有
      Some(v) if v == local => return Ok(start_heartbeat(key.into(), local, expire)),
      // Owned by others, try next
      // 被他人持有，尝试下一个
      _ => {
        last_occupied = id;
      }
    }
  }

  Err(Error::NoAvailablePid(MAX_PID))
}

/// Start heartbeat and return Pid
/// 启动心跳并返回 Pid
fn start_heartbeat(key: Box<[u8]>, local: [u8; 16], expire: Expiration) -> Pid {
  let id = pid_from_key(&key);
  let cancel = Arc::new(Notify::new());
  let notify = cancel.clone();

  tokio::spawn(async move {
    loop {
      tokio::select! {
        _ = notify.notified() => break,
        _ = tokio::time::sleep(HEARTBEAT) => {
          // Refresh expiration
          // 刷新过期时间
          if let Err(e) = R
            .set::<(), _, _>(&*key, &local[..], Some(expire.clone()), None, false)
            .await
          {
            log::error!("heartbeat set: {e}");
          }
        }
      }
    }
  });

  Pid { id, cancel }
}
