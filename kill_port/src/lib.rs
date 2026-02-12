#![cfg_attr(docsrs, feature(doc_cfg))]

use log::info;

pub fn kill_port(port: u16) {
  let my_pid = std::process::id();
  let mut retry = 0;
  loop {
    match listeners::get_processes_by_port(port) {
      Err(err) => {
        log::error!("kill_port {port} : {err}");
        break;
      }
      Ok(processes) => {
        let mut to_kill = 0;
        retry += 1;
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
                log::error!("kill process {} with {:?}: {err}", process.pid, signal);
              }
            }

            #[cfg(windows)]
            {
              use kill_tree::blocking::kill_tree;
              if let Err(err) = kill_tree(process.pid) {
                log::error!("Failed to kill process tree {}: {err}", process.pid);
              }
            }

            let sleep_ms = std::cmp::min(retry * 500, 1000);
            std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
          }
        }
        if to_kill == 0 {
          break;
        }
      }
    }
  }
}
