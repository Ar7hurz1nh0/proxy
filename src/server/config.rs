use std::{
  fs::File,
  io::{BufReader, BufWriter, Read, Write},
  time::{SystemTime, UNIX_EPOCH},
};

use once_cell::sync::Lazy;
use proxy_router::constants::{
  ConfigFile, Runtime, DEFAULT_THREAD_COUNT, SETTING_FILE_PATH,
};
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_string_pretty, Error};
use simplelog::{debug, error, info, trace, warn};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Address {
  pub port: u16,
  pub host: String,
}

pub trait ThreadType {
  type THREAD;
}

impl ThreadType for ConfigFile {
  type THREAD = Option<usize>;
}

impl ThreadType for Runtime {
  type THREAD = usize;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config<T: ThreadType> {
  pub separator: String,
  pub listen: Address,
  pub auth: String,
  pub threads: T::THREAD,
  pub concurrency: usize,
}

pub static DEFAULT_SETTINGS: Lazy<Config<ConfigFile>> = Lazy::new(|| Config {
  auth: String::from("CH4ng3M3!"),
  separator: String::from("\u{0000}"),
  listen: Address {
    port: 65535,
    host: String::from("0.0.0.0"),
  },
  threads: None,
  concurrency: 1024,
});

fn save_default() -> Result<(), ()> {
  let settings = to_string_pretty(&DEFAULT_SETTINGS.clone());
  match settings {
    | Ok(settings) => {
      let file = File::create(SETTING_FILE_PATH);
      match file {
        | Ok(file) => {
          let mut writer = BufWriter::new(file);
          match writer.write_all(settings.as_bytes()) {
            | Ok(_) => {
              info!("Settings file created!");
              return Result::Ok(());
            },
            | Err(e) => {
              error!(
                "Failed to write to settings file: {}",
                e
              );
              return Result::Err(());
            },
          }
        },
        | Err(e) => {
          error!("Failed to create settings file: {}", e);
          return Result::Err(());
        },
      }
    },
    | Err(e) => {
      error!(
        "Failed to serialize default settings: {}",
        e
      );
      return Result::Err(());
    },
  }
}

fn backup_settings(mut reader: BufReader<File>) -> Result<(), ()> {
  let mut settings: String = String::new();
  match reader.read_to_string(&mut settings) {
    | Ok(_) => {
      let backup_file: Result<File, std::io::Error> = File::create(format!(
        "{}-invalid-{}.json",
        SETTING_FILE_PATH.strip_suffix(".json").unwrap(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
      ));
      debug!(
        "Backup file name: {}",
        format!(
          "{}-invalid-{}.json",
          SETTING_FILE_PATH.strip_suffix(".json").unwrap(),
          SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        )
      );
      trace!("Backup file contents: {}", settings);
      match backup_file {
        | Ok(mut backup_file) => {
          match backup_file.write_all(&settings.as_bytes()) {
            | Ok(_) => {
              info!("Settings file backed up!");
              return Result::Ok(());
            },
            | Err(e) => {
              error!(
                "Failed to write to settings backup file: {}",
                e
              );
              return Result::Err(());
            },
          }
        },
        | Err(e) => {
          error!(
            "Failed to create settings backup file: {}",
            e
          );
          return Result::Err(());
        },
      }
    },
    | Err(e) => {
      error!("Failed to read settings file: {}", e);
      return Result::Err(());
    },
  }
}

fn file_to_runtime(config: Config<ConfigFile>) -> Config<Runtime> {
  let threads: usize = match config.threads {
    | Some(threads) => threads,
    | _ => match std::thread::available_parallelism() {
      | Ok(threads) => {
        warn!("Got null as number of threads, using system available threads ({threads} threads)");
        threads.into()
      },
      | Err(_) => {
        warn!("Unable to get system available threads, using default threads ({DEFAULT_THREAD_COUNT} threads)");
        DEFAULT_THREAD_COUNT
      },
    },
  };
  Config {
    auth: config.auth,
    concurrency: config.concurrency,
    listen: config.listen,
    separator: config.separator,
    threads,
  }
}

pub fn get_settings() -> Config<Runtime> {
  let settings: Config<ConfigFile> = DEFAULT_SETTINGS.clone();
  let file: Result<File, std::io::Error> = File::open(SETTING_FILE_PATH);
  match file {
    | Ok(file) => {
      let reader: BufReader<File> = BufReader::new(file);
      let settings_from_files: Result<Config<ConfigFile>, Error> =
        from_reader(reader);
      match settings_from_files {
        | Ok(settings_from_files) => {
          trace!("{:?}", settings_from_files);

          return file_to_runtime(settings_from_files);
        },
        | Err(e) => {
          error!("Failed to deserialize settings: {}", e);
          warn!("Using default settings");
          match backup_settings(BufReader::new(
            File::open(SETTING_FILE_PATH).unwrap(),
          )) {
            | Ok(_) => {
              save_default().unwrap();
            },
            | Err(_) => {
              error!("Failed to backup settings");
            },
          }
        },
      }
    },
    | Err(e) => {
      error!("Failed to open settings file: {}", e);
      warn!("Using default settings");
      save_default().unwrap();
    },
  }
  file_to_runtime(settings)
}
