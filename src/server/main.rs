mod config;
mod master;
mod slave;

use proxy::logging::{init_logger, LoggerSettings};

use clap::{value_parser, Arg, ArgAction, Command};
use signal_hook::{
  consts::{SIGINT, SIGTERM},
  iterator::Signals,
};
#[allow(unused_imports)]
use simplelog::{debug, error, info, trace, warn};
use std::{
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  thread,
  time::Duration, process::exit,
};

fn main() {
  let mut logger_settings = LoggerSettings {
    level: simplelog::LevelFilter::Info,
    file_level: simplelog::LevelFilter::Debug,
  };

  let level: simplelog::LevelFilter;
  let file_level: simplelog::LevelFilter;

  let before_help = format!(
    "{} {}\nLicense: {}\nSource: {}\nAuthors: {}",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_VERSION"),
    env!("CARGO_PKG_LICENSE"),
    env!("CARGO_PKG_HOMEPAGE"),
    env!("CARGO_PKG_AUTHORS").split(':').collect::<Vec<&str>>().join(", ")
  );

  let matches = Command::new(env!("CARGO_PKG_NAME"))
    .version(env!("CARGO_PKG_VERSION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .before_help(before_help)
    .name(env!("CARGO_PKG_NAME"))
    .about(env!("CARGO_PKG_DESCRIPTION"))
    .arg(
      Arg::new("trace")
        .long("trace")
        .num_args(0)
        .default_value("false")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .conflicts_with_all(&["debug", "error", "warn", "info", "off"])
        .help("Sets the logging level to trace"),
    )
    .arg(
      Arg::new("debug")
        .long("debug")
        .num_args(0)
        .action(ArgAction::SetTrue)
        .conflicts_with_all(&["trace", "error", "warn", "info", "off"])
        .help("Sets the logging level to debug"),
    )
    .arg(
      Arg::new("error")
        .long("error")
        .num_args(0)
        .action(ArgAction::SetTrue)
        .conflicts_with_all(&["trace", "debug", "warn", "info", "off"])
        .help("Sets the logging level to error"),
    )
    .arg(
      Arg::new("warn")
        .long("warn")
        .num_args(0)
        .action(ArgAction::SetTrue)
        .conflicts_with_all(&["trace", "debug", "error", "info", "off"])
        .help("Sets the logging level to warn"),
    )
    .arg(
      Arg::new("info")
        .long("info")
        .num_args(0)
        .action(ArgAction::SetTrue)
        .conflicts_with_all(&["trace", "debug", "error", "warn", "off"])
        .help("Sets the logging level to info (default)"),
    )
    .arg(
      Arg::new("off")
        .long("off")
        .num_args(0)
        .action(ArgAction::SetTrue)
        .conflicts_with_all(&["trace", "debug", "error", "warn", "info"])
        .help("Sets the logging level to off"),
    )
    .arg(
      Arg::new("trace-file")
        .long("trace-file")
        .num_args(0)
        .action(ArgAction::SetTrue)
        .conflicts_with("disable-log")
        .help("Sets the logging level to trace for the log file"),
    )
    .arg(
      Arg::new("disable-log")
        .long("disable-log")
        .num_args(0)
        .action(ArgAction::SetTrue)
        .conflicts_with("trace-file")
        .help("Disables the log file"),
    )
    .get_matches();

  if matches.get_flag("trace") {
    logger_settings.level = simplelog::LevelFilter::Trace;
    level = simplelog::LevelFilter::Trace;
  } else if matches.get_flag("debug") {
    logger_settings.level = simplelog::LevelFilter::Debug;
    level = simplelog::LevelFilter::Debug;
  } else if matches.get_flag("warn") {
    logger_settings.level = simplelog::LevelFilter::Warn;
    level = simplelog::LevelFilter::Warn;
  } else if matches.get_flag("error") {
    logger_settings.level = simplelog::LevelFilter::Error;
    level = simplelog::LevelFilter::Error;
  } else if matches.get_flag("off") {
    logger_settings.level = simplelog::LevelFilter::Off;
    level = simplelog::LevelFilter::Off;
  } else {
    level = simplelog::LevelFilter::Info;
  }

  if matches.get_flag("trace-file") {
    logger_settings.file_level = simplelog::LevelFilter::Trace;
    file_level = simplelog::LevelFilter::Trace;
  } else if matches.get_flag("disable-log") {
    logger_settings.file_level = simplelog::LevelFilter::Off;
    file_level = simplelog::LevelFilter::Off;
  } else {
    file_level = simplelog::LevelFilter::Debug;
  }

  init_logger(logger_settings);

  match level {
    | simplelog::LevelFilter::Trace => info!("TRACE calls logging to terminal"),
    | simplelog::LevelFilter::Debug => info!("DEBUG calls logging to terminal"),
    | simplelog::LevelFilter::Info => info!("INFO calls logging to terminal"),
    | simplelog::LevelFilter::Warn => info!("WARN calls logging to terminal"),
    | simplelog::LevelFilter::Error => info!("ERROR calls logging to terminal"),
    | simplelog::LevelFilter::Off => info!("Disabled logging to terminal"),
  }

  match file_level {
    | simplelog::LevelFilter::Trace => info!("TRACE calls logging to file"),
    | simplelog::LevelFilter::Debug => info!("DEBUG calls logging to file"),
    | simplelog::LevelFilter::Off => info!("Disabled logging to file"),
    | _ => (),
  }

  let atomic = Arc::new(AtomicBool::new(false));
  let mut signals: signal_hook::iterator::SignalsInfo =
    Signals::new(&[SIGINT, SIGTERM]).unwrap();

  let atomic_clone = Arc::clone(&atomic);
  thread::spawn(move || {
    for sig in signals.forever() {
      println!("");
      match sig {
        | SIGINT => warn!("Received SIGINT"),
        | SIGTERM => warn!("Received SIGTERM"),
        | _ => unreachable!(),
      }
      atomic_clone.store(true, Ordering::Relaxed);
    }
  });

  let config = config::get_settings();
  let listener = master::MasterListener::new(
    &config,
    Arc::clone(&atomic),
  );

  while !atomic.load(Ordering::Relaxed) {
    std::thread::sleep(Duration::from_millis(100));
  }
  let mut sleep: u16 = 0;
  while !listener.is_finished() && sleep < 5000 {
    std::thread::sleep(Duration::from_millis(100));
    sleep += 100;
  }
  exit(0);
}
