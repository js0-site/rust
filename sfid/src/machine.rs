use fred::{
  interfaces::{ClientLike, HashesInterface},
  types::{CustomCommand, Value},
};
use xkv::R;

use crate::{EXPIRE_SEC, Error, HEARTBEAT_SEC, MAX_MACHINE_ID, Result, SFID_KEY};

/// Wrapper type for machine ID
/// 机器号包装类型
pub struct MachineId(pub u16);

impl std::ops::Deref for MachineId {
  type Target = u16;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

/// Get unique identifier for this machine
/// 获取本机唯一标识
fn local_id() -> String {
  osid::get().unwrap_or_else(|_| uuid::Uuid::new_v4().to_string()).to_owned()
}

// Async initialized machine ID
// 异步初始化的机器号
static_::init!(MACHINE_ID: MachineId {
  allocate_machine_id().await.map(MachineId)
});

/// Get the machine ID (must call xboot::init().await first)
/// 获取机器号（必须先调用 xboot::init().await）
pub fn machine_id() -> u16 {
  **MACHINE_ID
}

/// Allocate a machine ID from Redis
/// 从 Redis 分配机器号
async fn allocate_machine_id() -> Result<u16> {
  let local = local_id();
  let max = MAX_MACHINE_ID as u64;

  // Generate random starting point
  // 生成随机起始点
  let start = (rand::random::<u64>() % max) as u16;

  for i in 0..MAX_MACHINE_ID {
    let id = (start + i) % MAX_MACHINE_ID;
    let field = id.to_string();

    // Try to claim this ID using HSETNX
    // 尝试使用 HSETNX 认领此 ID
    let set: i64 = R.hsetnx(SFID_KEY, &field, &local).await?;

    if set == 1 {
      // Successfully claimed, set expiration
      // 认领成功，设置过期时间
      set_field_expire(&field).await?;
      spawn_heartbeat(id, local);
      return Ok(id);
    }

    // Check if we already own this ID
    // 检查是否已经拥有此 ID
    let owner: Option<String> = R.hget(SFID_KEY, &field).await?;
    if owner.as_ref() == Some(&local) {
      // Refresh expiration
      // 刷新过期时间
      set_field_expire(&field).await?;
      spawn_heartbeat(id, local);
      return Ok(id);
    }
  }

  Err(Error::NoAvailableMachineId(MAX_MACHINE_ID))
}

/// Set expiration on hash field using HEXPIRE (Redis 7.4+)
/// 使用 HEXPIRE 设置哈希字段过期时间（Redis 7.4+）
async fn set_field_expire(field: &str) -> Result<()> {
  let _: Vec<i64> = R
    .custom(
      CustomCommand::new("HEXPIRE", SFID_KEY.as_bytes(), false),
      vec![
        Value::from(SFID_KEY),
        Value::from(EXPIRE_SEC.to_string()),
        Value::from("FIELDS"),
        Value::from("1"),
        Value::from(field),
      ],
    )
    .await?;
  Ok(())
}

/// Spawn heartbeat task to keep machine ID alive
/// 启动心跳任务保持机器号存活
fn spawn_heartbeat(id: u16, local: String) {
  tokio::spawn(async move {
    let field = id.to_string();
    let interval = std::time::Duration::from_secs(HEARTBEAT_SEC);

    loop {
      tokio::time::sleep(interval).await;

      // Refresh the value and expiration
      // 刷新值和过期时间
      if let Err(e) = R.hset::<(), _, _>(SFID_KEY, [(&field, &local)]).await {
        log::error!("heartbeat hset failed: {e}");
        continue;
      }

      if let Err(e) = set_field_expire(&field).await {
        log::error!("heartbeat expire failed: {e}");
      }
    }
  });
}
