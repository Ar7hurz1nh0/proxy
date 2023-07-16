use std::{
  fs::File,
  io::{BufReader, BufWriter, Read, Write},
  path::Path,
  time::{SystemTime, UNIX_EPOCH},
};

use once_cell::sync::Lazy;
use proxy::{
  constants::{CONFIG_FILE_EXT, CONFIG_FILE_NAME, CONFIG_FILE_PATH},
  utils::{ConfigFile, Runtime},
};
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_string_pretty, Error};
use simplelog::{debug, error, info, trace, warn};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Address {
  pub port: u16,
  pub addr: String,
}

pub trait ConfigType {
  type Auth;
  type Separator;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum ArrOrStr {
  STRING(String),
  ARR(Vec<u8>),
}

impl ConfigType for ConfigFile {
  type Auth = ArrOrStr;
  type Separator = ArrOrStr;
}

impl ConfigType for Runtime {
  type Auth = Vec<u8>;
  type Separator = Vec<u8>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config<C: ConfigType> {
  pub separator: C::Separator,
  pub listen: Address,
  pub auth: C::Auth,
}

pub static DEFAULT_SETTINGS: Lazy<Config<ConfigFile>> = Lazy::new(|| Config {
  auth: ArrOrStr::STRING(String::from("CH4ng3M3!")),
  separator: ArrOrStr::STRING(String::from("\u{0000}")),
  listen: Address {
    port: 65535,
    addr: String::from("0.0.0.0"),
  },
});

fn save_default() -> Result<(), ()> {
  let settings = to_string_pretty(&DEFAULT_SETTINGS.clone());
  if !CONFIG_FILE_PATH.is_empty() && !Path::new(&CONFIG_FILE_PATH).exists() {
    std::fs::create_dir(&CONFIG_FILE_PATH).unwrap();
  }
  let filename =
    format!("{CONFIG_FILE_PATH}{CONFIG_FILE_NAME}.server{CONFIG_FILE_EXT}");
  match settings {
    | Ok(settings) => {
      let file = File::create(filename);
      match file {
        | Ok(file) => {
          let mut writer = BufWriter::new(file);
          match writer.write_all(settings.as_bytes()) {
            | Ok(_) => {
              info!("Settings file created!");
              return Result::Ok(());
            },
            | Err(e) => {
              error!("Failed to write to settings file: {e}");
              return Result::Err(());
            },
          }
        },
        | Err(e) => {
          error!("Failed to create settings file: {e}");
          return Result::Err(());
        },
      }
    },
    | Err(e) => {
      error!("Failed to serialize default settings: {e}");
      return Result::Err(());
    },
  }
}

fn backup_settings(mut reader: BufReader<File>) -> Result<(), ()> {
  let mut settings: String = String::new();
  let filename = format!("{CONFIG_FILE_PATH}{CONFIG_FILE_NAME}.server");
  let filename = format!(
    "{filename}-invalid-{}.json",
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
  );
  match reader.read_to_string(&mut settings) {
    | Ok(_) => {
      debug!("Backup file name: {filename}");
      let backup_file: Result<File, std::io::Error> = File::create(filename);
      trace!("Backup file contents: {}", settings);
      match backup_file {
        | Ok(mut backup_file) => {
          match backup_file.write_all(&settings.as_bytes()) {
            | Ok(_) => {
              info!("Settings file backed up!");
              return Result::Ok(());
            },
            | Err(e) => {
              error!("Failed to write to settings backup file: {e}");
              return Result::Err(());
            },
          }
        },
        | Err(e) => {
          error!("Failed to create settings backup file: {e}");
          return Result::Err(());
        },
      }
    },
    | Err(e) => {
      error!("Failed to read settings file: {e}");
      return Result::Err(());
    },
  }
}

fn file_to_runtime(config: Config<ConfigFile>) -> Config<Runtime> {
  let auth: Vec<u8> = match config.auth {
    | ArrOrStr::STRING(auth) => auth.into_bytes(),
    | ArrOrStr::ARR(auth) => auth,
  };
  let separator: Vec<u8> = match config.separator {
    | ArrOrStr::STRING(separator) => separator.into_bytes(),
    | ArrOrStr::ARR(separator) => separator,
  };
  Config {
    auth,
    separator,
    listen: config.listen,
  }
}

pub fn get_settings() -> Config<Runtime> {
  let settings: Config<ConfigFile> = DEFAULT_SETTINGS.clone();
  let filename =
    format!("{CONFIG_FILE_PATH}{CONFIG_FILE_NAME}.server{CONFIG_FILE_EXT}");
  let file: Result<File, std::io::Error> = File::open(&filename);
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
          error!("Failed to deserialize settings: {e}");
          warn!("Using default settings");
          match backup_settings(BufReader::new(
            File::open(filename).unwrap(),
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
      error!("Failed to open config file: {e}");
      warn!("Using default settings");
      save_default().unwrap();
    },
  }
  file_to_runtime(settings)
}
