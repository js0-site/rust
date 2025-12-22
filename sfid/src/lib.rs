#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
mod machine;
mod snowflake;

pub use error::{Error, Result};
pub use machine::{MACHINE_ID, MachineId, machine_id};
pub use snowflake::Snowflake;

/// Redis hash key for machine ID allocation
/// Redis 哈希键，用于机器号分配
pub const SFID_KEY: &str = "sfid";

/// Heartbeat interval in seconds (15 minutes)
/// 心跳间隔（秒），15分钟
pub const HEARTBEAT_SEC: u64 = 15 * 60;

/// Expiration time in seconds (1 hour)
/// 过期时间（秒），1小时
pub const EXPIRE_SEC: u64 = 60 * 60;

/// Maximum machine ID (10 bits = 1024)
/// 最大机器号（10位 = 1024）
pub const MAX_MACHINE_ID: u16 = 1024;
