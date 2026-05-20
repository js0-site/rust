#![cfg_attr(docsrs, feature(doc_cfg))]
use std::{
  future::Future,
  marker::PhantomData,
  pin::Pin,
  task::{Context, Poll},
  time::Duration as StdDuration,
};

use coarsetime::{Duration, Instant};
pub use futures::{Stream, StreamExt};
use tokio::time::{Sleep, sleep};

/// Staggered race executor / 阶梯式竞赛执行器
pub struct Race<'a, A, T, E, G, Fut, I> {
  pub ing: Vec<(A, Pin<Box<Fut>>)>,
  step: Duration,
  run: G,
  args: I,
  next_run: Instant,
  timer: Option<Pin<Box<Sleep>>>,
  is_end: bool,
  _phantom: PhantomData<&'a (A, T, E)>,
}

impl<'a, A, T, E, G, Fut, I> Race<'a, A, T, E, G, Fut, I>
where
  A: Send + Unpin + 'a,
  T: Send + 'a,
  E: Send + 'a,
  G: Fn(&A) -> Fut + Send + 'a,
  Fut: Future<Output = Result<T, E>> + Send + 'a,
  I: Iterator<Item = A> + Send + 'a,
{
  pub fn new<II>(step: StdDuration, run: G, args_li: II) -> Self
  where
    II: IntoIterator<Item = A, IntoIter = I>,
  {
    Self {
      step: step.into(),
      run,
      args: args_li.into_iter(),
      ing: Vec::with_capacity(8),
      next_run: Instant::now(),
      timer: None,
      is_end: false,
      _phantom: PhantomData,
    }
  }
}

impl<'a, A, T, E, G, Fut, I> Stream for Race<'a, A, T, E, G, Fut, I>
where
  A: Send + Unpin + 'a,
  T: Send + 'a,
  E: Send + 'a,
  G: Fn(&A) -> Fut + Send + Unpin + 'a,
  Fut: Future<Output = Result<T, E>> + Send + 'a,
  I: Iterator<Item = A> + Send + Unpin + 'a,
{
  type Item = (A, Result<T, E>);

  fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    let this = self.as_mut().get_mut();

    // Try to start next task if it's time / 尝试启动下一个任务如果时间到了
    if !this.is_end {
      let now = Instant::now();
      while now >= this.next_run {
        if let Some(arg) = this.args.next() {
          let task = Box::pin((this.run)(&arg));
          this.ing.push((arg, task));
          this.next_run += this.step;
        } else {
          this.is_end = true;
          this.timer = None;
          break;
        }
      }

      // Set timer for next task start / 设置下一个任务启动的定时器
      if !this.is_end && this.timer.is_none() {
        let delay = if this.next_run > now {
          StdDuration::from(this.next_run.duration_since(now))
        } else {
          StdDuration::ZERO
        };
        this.timer = Some(Box::pin(sleep(delay)));
      }
    }

    // Poll timer to register waker / 轮询定时器以注册唤醒器
    if let Some(timer) = this.timer.as_mut()
      && timer.as_mut().poll(cx).is_ready()
    {
      this.timer = None;
      cx.waker().wake_by_ref();
    }

    // Poll all running tasks and return first completed / 轮询所有运行中的任务，返回首个完成的
    for i in (0..this.ing.len()).rev() {
      if let Poll::Ready(result) = this.ing[i].1.as_mut().poll(cx) {
        let (arg, _) = this.ing.swap_remove(i);
        return Poll::Ready(Some((arg, result)));
      }
    }

    // If no tasks running and args exhausted, we're done / 如果没有运行中的任务且参数耗尽，则完成
    if this.ing.is_empty() && this.is_end {
      return Poll::Ready(None);
    }

    Poll::Pending
  }
}
