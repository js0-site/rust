use std::sync::atomic::Ordering;

use fred::interfaces::HashesInterface;
use xkv::R;

#[cfg(debug_assertions)]
use crate::LAST_ID;
use crate::{Error, KVID_KEY, KvId, PRELOAD_SEC, Result, STEP_MAX, STEP_MIN, Seg, Slow};

#[cfg(debug_assertions)]
fn debug_check(id: u64) {
  let last = LAST_ID.swap(id, Ordering::Relaxed);
  if last > 0 && id != last + 1 {
    dbg!("id not continuous", last, id);
  }
}

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
        #[cfg(debug_assertions)]
        debug_check(nid);
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
      // 间隔为0说明消费极快，步长翻倍 / zero elapsed means high load, double step
      return (slow.step * 2).min(STEP_MAX);
    }
    (slow.step * PRELOAD_SEC / elapsed).clamp(STEP_MIN, STEP_MAX)
  }

  pub(crate) async fn fetch(&self, step: u64) -> Result<Seg> {
    let incr = i64::try_from(step).map_err(|_| Error::StepOverflow(step))?;
    let max = R
      .hincrby::<u64, _, _>(KVID_KEY, self.name.as_str(), incr)
      .await?;
    #[cfg(debug_assertions)]
    dbg!(&self.name, step, max);
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
