#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{cmp, collections::HashSet, process, thread, time::Duration};

use log::{error, info};

pub fn kill_port(port: u16) {
  if port == 0 {
    return;
  }
  let my_pid = process::id();
  let mut retry = 0;
  loop {
    match listeners::get_all() {
      Err(err) => {
        error!("kill_port {port} : {err}");
        break;
      }
      Ok(all_listeners) => {
        let mut to_kill = 0;
        retry += 1;

        let mut unique_pids = HashSet::new();
        let mut processes = Vec::new();
        for listener in all_listeners {
          if listener.socket.port() == port && unique_pids.insert(listener.process.pid) {
            processes.push(listener.process);
          }
        }

        for process in processes {
          if process.pid != my_pid {
            to_kill += 1;
            info!(
              "{my_pid} | {retry} | kill_port {port} → {} pid={}",
              process.name, process.pid,
            );

            // Kill the process / 杀死进程
            #[cfg(unix)]
            {
              use nix::{
                sys::signal::{Signal, kill},
                unistd::Pid,
              };
              // Use SIGKILL after 10 retries for forceful termination / 重试超过10次后使用SIGKILL强制终止
              let signal = if retry > 10 {
                Signal::SIGKILL
              } else {
                Signal::SIGTERM
              };
              if let Err(err) = kill(Pid::from_raw(process.pid as i32), signal) {
                error!("kill process {} with {:?}: {err}", process.pid, signal);
              }
            }

            #[cfg(windows)]
            {
              use kill_tree::blocking::kill_tree;
              if let Err(err) = kill_tree(process.pid) {
                error!("Failed to kill process tree {}: {err}", process.pid);
              }
            }

            let sleep_ms = cmp::min(retry * 500, 1000);
            thread::sleep(Duration::from_millis(sleep_ms));
          }
        }
        if to_kill == 0 {
          break;
        }
      }
    }
  }
}
