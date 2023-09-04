use proxy::utils::{PacketType, Runtime, Server};
use simplelog::{debug, error, info, trace, warn};
use std::{
  collections::HashMap,
  io::Error,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  thread,
  time::Duration,
};
use tokio::{
  io::AsyncReadExt,
  net::{TcpListener, TcpStream},
  runtime::{self, Runtime as TokioRuntime},
  sync::Mutex,
  time::sleep,
};
use uuid::Uuid;

use crate::slave::{Address, SenderPacket, ServerConfig, SlaveListener};

#[derive(Clone)]
pub struct MasterListener {
  config: crate::config::Config<Runtime>,
  was_authed: bool,
  connections: Arc<Mutex<HashMap<Uuid, SenderPacket>>>,
}

impl MasterListener {
  pub fn new(
    config: &crate::config::Config<Runtime>, drop_handler: Arc<AtomicBool>,
  ) -> std::thread::JoinHandle<()> {
    let mut master = MasterListener {
      config: config.to_owned(),
      was_authed: false,
      connections: Arc::new(Mutex::new(HashMap::new())),
    };
    thread::spawn(move || {
      master.server(drop_handler);
    })
  }

  #[tokio::main]
  async fn server(&mut self, drop_handler: Arc<AtomicBool>) {
    todo!("Implement master server");
  }
}
