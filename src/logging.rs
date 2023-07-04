use std::{
  fs::{metadata, rename, File},
  path::Path,
};

use chrono::{DateTime, Local, NaiveDateTime};
use simplelog::{
  Color, ColorChoice, CombinedLogger, ConfigBuilder, Level, LevelFilter,
  TermLogger, TerminalMode, WriteLogger,
};

use super::constants::{LOG_FILE, LOG_PATH};

pub struct LoggerSettings {
  pub level: LevelFilter,
  pub file_level: LevelFilter,
}

pub fn init_logger(settings: LoggerSettings) -> () {
  let config = ConfigBuilder::new()
    .set_level_color(Level::Trace, Some(Color::Magenta))
    .set_level_color(Level::Debug, Some(Color::Cyan))
    .set_level_color(Level::Info, Some(Color::White))
    .set_level_color(Level::Warn, Some(Color::Yellow))
    .set_level_color(Level::Error, Some(Color::Red))
    .build();

  if !Path::new(&LOG_PATH).exists() {
    std::fs::create_dir(&LOG_PATH).unwrap();
  }

  let latest_log_path: String = format!("{}/{}", &LOG_PATH, &LOG_FILE);
  if Path::new(&latest_log_path).exists() {
    let metadata = metadata(&latest_log_path).unwrap();
    let created: DateTime<Local> = metadata.created().unwrap().into();
    let datetime: NaiveDateTime = created.naive_local();
    rename(
      &latest_log_path,
      format!(
        "{}/{}.log",
        &LOG_PATH,
        datetime.format("%Y-%m-%d-%H-%M-%S").to_string()
      ),
    )
    .unwrap();
  }

  CombinedLogger::init(vec![
    TermLogger::new(
      settings.level,
      config.clone(),
      TerminalMode::Mixed,
      ColorChoice::Auto,
    ),
    WriteLogger::new(
      settings.file_level,
      config,
      File::create(&latest_log_path).unwrap(),
    ),
  ])
  .unwrap();
}
