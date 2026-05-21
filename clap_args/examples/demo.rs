use clap_args::{ArgAction, arg};

fn main() -> aok::Result<()> {
  if let Some(matches) = clap_args::parse!(|cmd| {
    cmd
      .arg(arg!(-b --bind [BIND] "http proxy bind address").default_value("0.0.0.0:15080"))
      .arg(arg!(-p --port <PORT> "listen port"))
      .arg(arg!(-d --debug "enable debug mode").action(ArgAction::SetTrue))
  }) {
    let bind: &String = matches.get_one("bind").unwrap();
    println!("bind: {bind}");

    if let Some(port) = matches.get_one::<String>("port") {
      println!("port: {port}");
    }

    if matches.get_flag("debug") {
      println!("debug mode enabled");
    }
  }
  Ok(())
}
