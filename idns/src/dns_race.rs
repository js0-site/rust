use std::{fmt::Debug, time::Duration};

use pick_fast::PickFast;
use race::{Race, StreamExt};

use crate::{Answer, QType, Query};

pub struct DnsRace<Q: Query> {
  pub li: PickFast<Q>,
}

impl<Q: Query> DnsRace<Q> {
  pub fn new(li: impl IntoIterator<Item = Q>) -> Self {
    Self {
      li: PickFast::new(li),
    }
  }
}

struct Task {
  pub index: usize,
  pub start: u64,
}

impl<Q: Query + Debug> Query for DnsRace<Q> {
  type Error = Q::Error;

  async fn answer_li(&self, qtype: QType, name: &str) -> Result<Option<Vec<Answer>>, Q::Error> {
    let mut race = Race::new(
      Duration::from_millis(500),
      |task: &Task| {
        let index = task.index;
        async move { self.li.li[index].answer_li(qtype, name).await }
      },
      self.li.iter().map(|i| Task {
        index: i.0,
        start: ts_::milli(),
      }),
    );

    while let Some((task, res)) = race.next().await {
      match res {
        Ok(res) => {
          // Update weight based on latency / 根据延迟更新权重
          let latency = (ts_::milli() - task.start) as u32;
          self.li.set(task.index, latency);
          // log::debug!(
          //   "success {:?} {name} latency={latency}ms",
          //   &*self.li.li[task.index]
          // );

          // Mark remaining tasks as failed / 将剩余任务标记为失败
          for (t, _) in race.ing {
            self.li.failed(t.index);
            // log::debug!("cancel {:?} {name}", &*self.li.li[t.index]);
          }

          return Ok(res);
        }
        Err(err) => {
          self.li.failed(task.index);
          log::error!("{:?} {name} : {err}", *self.li.li[task.index]);
        }
      }
    }

    Ok(None)
  }
}
