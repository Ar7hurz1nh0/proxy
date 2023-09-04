mod config;
// mod socket; // unused atm until I want to add UDP support
mod tunnel;

use crate::tunnel::Tunnel;
use std::{
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
  thread,
};

use clap::{value_parser, Arg, ArgAction, Command};
use proxy::logging::{init_logger, LoggerSettings};
use signal_hook::{
  consts::{SIGINT, SIGTERM},
  iterator::Signals,
};
#[allow(unused_imports)]
use simplelog::{debug, error, info, trace, warn};

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
        | _ => warn!("Unexpected signal"),
      }
      atomic_clone.store(true, Ordering::SeqCst);
    }
  });

  let config = config::get_settings();
  // socket::connect(&config, Arc::clone(&atomic));

  let tunnels: Arc<Mutex<Vec<Tunnel>>> = Arc::new(Mutex::new(Vec::new()));

  for target in config.targets {
    let tunnel = config.ssh_config.create_tunnel(
      target.source_port,
      target.address.to_owned(),
      target.target_port,
    );
    match tunnel {
      | Ok(tunnel) => {
        tunnels.lock().unwrap().push(tunnel);
        info!(
          "Tunnel {}:{} <- {}:{} created!",
          target.address,
          target.source_port,
          &config.ssh_config.host,
          target.target_port
        )
      },
      | Err(err) => {
        error!(
          "Error creating {}:{} <- {}:{} tunnel: {}",
          target.address,
          target.source_port,
          &config.ssh_config.host,
          target.target_port,
          err
        );
      },
    }
  }

  while &tunnels.lock().unwrap().len() > &0_usize {
    if atomic.load(Ordering::Relaxed) {
      warn!("Stopping tunnel resurrection service!");
      break;
    }
    let tunnels_arc = Arc::clone(&tunnels);
    trace!("Acquiring tunnels lock");
    let mut tunnels_lock = tunnels_arc.lock().unwrap();
    trace!("Tunnels lock acquired");
    for tunnel in tunnels_lock.iter_mut() {
      match tunnel.proccess.try_wait() {
        | Ok(Some(status)) => {
          if let Some(status) = status.code() {
            if status > 0 {
              debug!("Tunnel has died, resurrecting");
              let tunnel = &config.ssh_config.create_tunnel(
                tunnel.source_port, tunnel.source_host.to_owned(), tunnel.target_port,
              );
              match tunnel {
                | Ok(tunnel) => debug!(
                  "{}:{} <- {}:{} tunnel resurrected",
                  tunnel.source_host,
                  tunnel.source_port,
                  &config.ssh_config.host,
                  tunnel.target_port
                ),
                | Err(err) => error!("Error while resurrecting tunnel: {err}"),
              }
            } else {
              warn!("Tunnel has terminated, not resurrecting");
              Arc::clone(&tunnels)
                .lock()
                .unwrap()
                .retain(|t| t.proccess.id() != tunnel.proccess.id())
            }
          }
        },
        | Ok(None) => (),
        | Err(err) => error!("Error checking tunnel: {}", err),
      }
    }
    thread::sleep(std::time::Duration::from_millis(100));
  }

  let tunnels = Arc::clone(&tunnels);

  for tunnel in tunnels.lock().unwrap().iter_mut() {
    match tunnel.proccess.kill() {
      | Ok(_) => info!(
        "{}:{} <- {}:{} tunnel killed!",
        tunnel.source_host,
        tunnel.source_port,
        &config.ssh_config.host,
        tunnel.target_port
      ),
      | Err(err) => error!("Error killing tunnel: {}", err),
    }
  }
}
