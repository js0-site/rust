use std::{
  fs,
  io::{self, Read, Write},
  process::exit,
};

use add_space::{add_space, Result};
use clap::Parser;

/// CLI arguments structure.
/// CLI 参数结构体
#[derive(Parser)]
#[command(name = "add_space", author, version, about, long_about = None)]
struct Cli {
  /// The path to the file to process, or stdin if not provided
  path: Option<String>,

  /// Write the output back to the file
  #[arg(short, long)]
  write: bool,
}

/// Process full string content line by line.
/// 逐行处理完整字符串内容
fn process_content(content: &str) -> String {
  let mut result = String::with_capacity(content.len());
  let mut lines = content.lines();
  if let Some(line) = lines.next() {
    result.push_str(&add_space(line));
    for line in lines {
      result.push('\n');
      result.push_str(&add_space(line));
    }
  }
  result
}

fn main() -> Result<()> {
  let cli = Cli::parse();

  let (content, from_stdin) = if let Some(path) = &cli.path {
    (fs::read_to_string(path)?, false)
  } else {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    (buffer, true)
  };

  if cli.write {
    if from_stdin {
      eprintln!("Error: cannot use --write with stdin.");
      exit(1);
    }
    if let Some(path) = &cli.path {
      let output = process_content(&content);
      fs::write(path, output)?;
      println!("File {path} has been updated.");
    }
  } else {
    let mut stdout = io::stdout().lock();
    let output = process_content(&content);
    stdout.write_all(output.as_bytes())?;
  }

  Ok(())
}
