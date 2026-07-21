use std::{
  fs,
  io::{self, Read},
  process::exit,
};

use add_space::add_space;
use clap::Parser;

#[derive(Parser)]
#[command(name = "add_space", author, version, about, long_about = None)]
struct Cli {
  /// The path to the file to process, or stdin if not provided
  path: Option<String>,

  /// Write the output back to the file
  #[arg(short, long)]
  write: bool,
}

fn main() -> io::Result<()> {
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
      let mut new_content = String::with_capacity(content.len());
      let mut lines = content.lines();
      if let Some(line) = lines.next() {
        new_content.push_str(&add_space(line));
        for line in lines {
          new_content.push('\n');
          new_content.push_str(&add_space(line));
        }
      }
      fs::write(path, new_content)?;
      println!("File {} has been updated.", path);
    }
  } else {
    use std::io::Write;
    let mut stdout = io::stdout().lock();
    let mut lines = content.lines();
    if let Some(line) = lines.next() {
      write!(stdout, "{}", add_space(line))?;
      for line in lines {
        write!(stdout, "\n{}", add_space(line))?;
      }
    }
  }

  Ok(())
}
