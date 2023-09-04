use crate::config::SSHConfig;
use std::{io::Error, process::Child};

pub struct Tunnel {
  pub target_port: u16,
  pub source_port: u16,
  pub source_host: String,
  pub proccess: Child,
}

impl SSHConfig {
  pub fn create_tunnel(
    &self, source_port: u16, source_host: String, target_port: u16,
  ) -> Result<Tunnel, Error> {
    let host = self.host.clone();
    let port = self.port.clone();
    let user = self.user.clone();
    let key_path = self.key_path.clone();
    let args = self.aditional_args.clone();
    let mut command = std::process::Command::new("ssh");
    command
      .arg("-N")
      .arg("-R")
      .arg(format!(
        "{target_port}:{source_host}:{source_port}"
      ))
      .arg(format!("{user}@{host}"))
      .arg("-p")
      .arg(format!("{port}"))
      .arg("-i")
      .arg(key_path)
      .arg("-g")
      .arg("-o")
      .arg(format!("ServerAliveInterval={}", 30))
      .arg("-o")
      .arg(format!("ServerAliveCountMax={}", 3))
      .arg("-o")
      .arg(format!(
        "ExitOnForwardFailure={}",
        "yes"
      ));

    for arg in args {
      command.arg(arg);
    }

    let process = command
      .stdin(std::process::Stdio::piped())
      .stdout(std::process::Stdio::inherit())
      .stderr(std::process::Stdio::piped())
      .spawn();

    match process {
      | Ok(proccess) => Ok(Tunnel {
        source_host,
        source_port,
        target_port,
        proccess,
      }),
      | Err(err) => Err(err),
    }
  }
}
