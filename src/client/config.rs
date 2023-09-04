use std::{
  fs::File,
  io::{BufReader, BufWriter, Read, Write},
  path::Path,
};

use ariadne::{Color, ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use once_cell::sync::Lazy;
use proxy::{
  constants::{CONFIG_FILE_EXT, CONFIG_FILE_NAME, CONFIG_FILE_PATH},
  utils::{ConfigFile, Runtime},
};
use serde::{Deserialize, Serialize};
use serde_json::{error::Category, from_reader, to_string_pretty, Error};
use simplelog::{error, info, trace, warn};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SSHTarget {
  pub address: String,
  pub source_port: u16,
  pub target_port: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum ArrOrStr {
  STRING(String),
  ARR(Vec<u8>),
}

pub trait ConfigType {
  type Auth;
  type Separator;
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
pub struct SSHConfig {
  pub server_alive_interval: Option<u32>,
  pub server_alive_count_max: Option<u32>,
  pub exit_on_forward_failure: Option<bool>,
  pub aditional_args: Vec<String>,
  pub host: String,
  pub port: u16,
  pub user: String,
  pub key_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config<C: ConfigType> {
  pub targets: Vec<SSHTarget>,
  pub separator: C::Separator,
  pub auth: C::Auth,
  pub ssh_config: SSHConfig,
}

pub static DEFAULT_SETTINGS: Lazy<Config<ConfigFile>> = Lazy::new(|| Config {
  auth: ArrOrStr::STRING(String::from("CH4ng3M3!")),
  separator: ArrOrStr::STRING(String::from("\u{0000}")),

  targets: vec![
    SSHTarget {
      address: String::from("0.0.0.0"),
      source_port: 0,
      target_port: 0,
    },
    SSHTarget {
      address: String::from("localhost"),
      source_port: 0,
      target_port: 0,
    },
  ],
  ssh_config: SSHConfig {
    server_alive_interval: Some(30),
    server_alive_count_max: Some(3),
    exit_on_forward_failure: Some(true),
    aditional_args: [].to_vec(),
    host: String::from("localhost"),
    port: 22,
    user: String::from("ubuntu"),
    key_path: String::from("~/.ssh/id_rsa"),
  },
});

fn save_default() -> Result<(), ()> {
  let settings = to_string_pretty(&DEFAULT_SETTINGS.clone());
  if !Path::new(&CONFIG_FILE_PATH).exists() && !CONFIG_FILE_PATH.is_empty() {
    std::fs::create_dir(&CONFIG_FILE_PATH).unwrap();
  }
  let filename =
    format!("{CONFIG_FILE_PATH}{CONFIG_FILE_NAME}.client{CONFIG_FILE_EXT}");
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
          error!("Failed to create settings file: {}", e);
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

fn file_to_runtime(config: Config<ConfigFile>) -> Config<Runtime> {
  let auth: Vec<u8> = match config.auth {
    | ArrOrStr::STRING(auth) => auth.into_bytes(),
    | ArrOrStr::ARR(auth) => auth,
  };
  let separator: Vec<u8> = match config.separator {
    | ArrOrStr::STRING(separator) => separator.into_bytes(),
    | ArrOrStr::ARR(separator) => separator,
  };
  let mut ssh_config: SSHConfig = config.ssh_config;

  if ssh_config.exit_on_forward_failure.is_none() {
    ssh_config.exit_on_forward_failure = Some(true);
  }

  if ssh_config.server_alive_interval.is_none() {
    ssh_config.server_alive_interval = Some(30);
  }

  if ssh_config.server_alive_count_max.is_none() {
    ssh_config.server_alive_count_max = Some(3);
  }

  Config {
    auth,
    separator,
    targets: config.targets,
    ssh_config,
  }
}

fn count_characters_until_position(
  text: &str, target_line: usize, target_column: usize,
) -> usize {
  let lines: Vec<&str> = text.lines().collect();
  let mut total_count = 0;

  for (line_number, line) in lines.iter().enumerate() {
    if line_number >= target_line - 1 {
      total_count += target_column - 1; // Subtract 1 since indexing starts from 0
      break;
    }

    total_count += line.len() + 1; // Add 1 to account for the newline character
  }

  total_count
}

/// Returns the erroneous_type, expected_type, marker_size, marker_offset, respectivelly
fn get_error_info(
  error: &Error, expected_color: Color,
) -> Option<(Option<String>, String, usize, usize)> {
  let error = error.to_string();
  if error.starts_with("invalid type: ") {
    let buffer = error.split_once(": ").unwrap().1;
    if buffer.starts_with("null") {
      let mut expected_type = buffer.split_once("expected ").unwrap().1;
      if expected_type.starts_with("a") {
        expected_type = expected_type.split_once("a ").unwrap().1;
      }
      let expected_type = expected_type.split_once(" ").unwrap().0;
      let erroneous_type = "null";
      let marker_size = 4;
      let marker_offset = 1;
      return Some((
        Some(format!("unexpected {erroneous_type}")),
        format!(
          "replace highlighted code with expected type ({})",
          expected_type.fg(expected_color)
        ),
        marker_size - marker_offset,
        marker_offset,
      ));
    } else {
      let (erroneous_type, buffer) = buffer.split_once(" ").unwrap();
      let (marker_size, buffer) = buffer.split_once(",").unwrap();
      let mut expected_type = buffer.split_once("expected ").unwrap().1;
      if expected_type.starts_with("a") {
        expected_type = expected_type.split_once("a ").unwrap().1;
      }
      let expected_type = expected_type.split_once(" ").unwrap().0;
      if marker_size.starts_with("\"") {
        let marker_size = marker_size.len();
        let marker_offset = 1;
        return Some((
          Some(format!("unexpected {erroneous_type}")),
          format!(
            "replace highlighted code with expected type ({})",
            expected_type.fg(expected_color)
          ),
          marker_size - marker_offset,
          marker_offset,
        ));
      } else if marker_size.starts_with("`") {
        let marker_size = marker_size.replace("`", "").replace("`", "").len();
        let marker_offset = 0;
        return Some((
          Some(format!("unexpected {erroneous_type}")),
          format!(
            "replace highlighted code with expected type ({})",
            expected_type.fg(expected_color)
          ),
          marker_size - marker_offset,
          marker_offset,
        ));
      }
    }
  }
  return None;
}

pub fn get_settings() -> Config<Runtime> {
  let settings: Config<ConfigFile> = DEFAULT_SETTINGS.clone();
  let filename =
    format!("{CONFIG_FILE_PATH}{CONFIG_FILE_NAME}.client{CONFIG_FILE_EXT}");
  let file: Result<File, std::io::Error> = File::open(&filename);
  match file {
    | Ok(file) => {
      let reader: BufReader<File> = BufReader::new(file);
      let settings_from_files: Result<Config<ConfigFile>, Error> =
        from_reader(&mut reader.get_ref().try_clone().unwrap());
      match settings_from_files {
        | Ok(settings_from_files) => {
          trace!("{:?}", settings_from_files);
          return file_to_runtime(settings_from_files);
        },
        | Err(e) => {
          if e.classify() == Category::Io {
            info!("Creating config file");
            save_default().unwrap();
            info!("Config file created");
            std::process::exit(0);
          }
          error!("Failed to deserialize settings: {e}");
          let mut colors = ColorGenerator::new();
          let rnd1: u16 = rand::random();
          let rnd2: u16 = rand::random();
          let min = if rnd1 < rnd2 {
            rnd1
          } else {
            rnd2
          };
          let max = if rnd1 < rnd2 {
            rnd2
          } else {
            rnd1
          };
          for _ in min..max {
            colors.next();
          }
          let error_color = colors.next();
          let expected_color = colors.next();
          let file = File::open(&filename).unwrap();
          let mut reader = BufReader::new(file);
          let mut buf = String::new();
          let readable_error_type = match e.classify() {
            | Category::Data => "Invalid type",
            | Category::Eof => "Unexpected end of file",
            | Category::Syntax => "Invalid JSON syntax",
            | Category::Io => "IO error",
          };
          reader.read_to_string(&mut buf).unwrap();
          let error_info = get_error_info(&e, expected_color);
          if error_info.is_none() {
            std::process::exit(2);
          }
          let (erroneous_type, expected_type, marker_size, marker_offset) =
            error_info.unwrap();
          let error_start =
            count_characters_until_position(buf.as_str(), e.line(), e.column());
          let end = marker_offset + error_start;
          let start = error_start - marker_size;
          let mut report = Report::<(&str, std::ops::Range<usize>)>::build(
            ReportKind::Error,
            "config.client.json",
            start,
          )
          .with_code(24)
          .with_message(readable_error_type);

          if let Some(erroneous_type) = erroneous_type {
            report = report.with_label(
              Label::new(("config.client.json", start..end))
                .with_message(erroneous_type)
                .with_color(error_color),
            );
          }

          report
            .with_help(expected_type)
            .finish()
            .print((
              "config.client.json",
              Source::from(buf.as_str()),
            ))
            .unwrap();

          std::process::exit(2);
        },
      }
    },
    | Err(e) => {
      error!("Failed to open settings file: {e}");
      warn!("Using default settings");
      save_default().unwrap();
    },
  }
  file_to_runtime(settings)
}
