use fred::interfaces::HashesInterface;
use parking_lot::{Mutex, lock_api::RawMutex};
use xkv::R;

// 平均每10分钟请求请求一次kvrocks数据库
pub const FETCH_DURATION: u64 = 600;
pub const STEP_MAX: u64 = 1000000;

#[derive(Debug, Default)]
pub struct Inner {
  pub id: u64,
  pub max: u64,
  pub time: u64,
  pub step: u64,
  pub ts: u64,
}

pub struct KvId {
  pub name: &'static str,
  inner: Mutex<Inner>,
}

impl KvId {
  pub async fn next(&self) -> fred::interfaces::FredResult<u64> {
    let step;
    let pre_ts;
    {
      let mut kvid = self.inner.lock();
      if kvid.id > 0 && kvid.id < kvid.max {
        kvid.id += 1;
        return Ok(kvid.id);
      }
      step = kvid.step;
      pre_ts = kvid.ts;
    }
    let now = ts_::sec();
    let max = R.hincrby::<u64, _, _>("kvid", self.name, step as _).await?;
    let mut kvid = self.inner.lock();
    kvid.max = max;
    kvid.id = max - step + 1;
    if pre_ts > 0 && step < STEP_MAX {
      if now <= pre_ts {
        kvid.step = step * 2;
      } else {
        let cost = now - pre_ts;
        if cost > FETCH_DURATION {
          kvid.step = std::cmp::max(1, step / 2);
        } else {
          kvid.step = step * 2;
        }
      }
    }
    kvid.ts = now;
    Ok(kvid.id)
  }

  pub const fn new(name: &'static str) -> Self {
    Self {
      inner: Mutex::const_new(
        parking_lot::RawMutex::INIT,
        Inner {
          id: 0,
          max: 0,
          time: 0,
          step: 1,
          ts: 0,
        },
      ),
      name,
    }
  }
}
