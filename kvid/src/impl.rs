use std::sync::atomic::Ordering;

use fred::interfaces::HashesInterface;
use xkv::R;

use crate::{Error, KVID_KEY, KvId, PRELOAD_SEC, Result, STEP_MAX, STEP_MIN, Seg, Slow};

impl KvId {
  #[inline]
  pub(crate) fn try_next(&self) -> Option<u64> {
    loop {
      let id = self.fast.id.load(Ordering::Acquire);
      let max = self.fast.max.load(Ordering::Acquire);
      if id >= max {
        return None;
      }
      let nid = id + 1;
      if self
        .fast
        .id
        .compare_exchange_weak(id, nid, Ordering::AcqRel, Ordering::Relaxed)
        .is_ok()
      {
        return Some(nid);
      }
    }
  }

  // 根据上次步长和时间间隔计算新步长
  // calc new step based on prev step and elapsed time
  pub(crate) fn calc_step(slow: &Slow, now: u64) -> u64 {
    if slow.ts == 0 {
      return STEP_MIN;
    }
    let elapsed = now.saturating_sub(slow.ts);
    if elapsed == 0 {
      // 间隔为0说明消费极快，快速增长 / zero elapsed means high load, grow fast
      // 1→64 then double / 1→64 然后翻倍
      return if slow.step < 64 {
        64
      } else {
        (slow.step * 2).min(STEP_MAX)
      };
    }
    (slow.step * PRELOAD_SEC / elapsed).clamp(STEP_MIN, STEP_MAX)
  }

  pub(crate) async fn fetch(&self, step: u64) -> Result<Seg> {
    let incr = i64::try_from(step).map_err(|_| Error::StepOverflow(step))?;
    let max = R
      .hincrby::<u64, _, _>(KVID_KEY, self.name.as_str(), incr)
      .await?;
    Ok(Seg {
      id: max - step,
      max,
    })
  }

  #[inline]
  pub(crate) fn set_seg(&self, seg: Seg) {
    self.fast.id.store(seg.id, Ordering::Release);
    self.fast.max.store(seg.max, Ordering::Release);
  }
}
