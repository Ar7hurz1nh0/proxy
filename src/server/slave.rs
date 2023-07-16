use proxy::utils::Server;
use simplelog::{error, info, trace};
use std::{
  collections::HashMap,
  io::{Error, Read, Write},
  net::{TcpListener, TcpStream},
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  thread,
  time::Duration,
};
use tokio::{runtime::Runtime, sync::Mutex, time::sleep};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Address {
  pub port: u16,
  pub addr: String,
}

#[derive(Clone)]
pub struct ServerConfig {
  pub separator: Vec<u8>,
  pub listen: Address,
  pub threads: usize,
  pub concurrency: usize,
  pub main_socket: Arc<Mutex<TcpStream>>,
}

pub struct SenderPacket {
  pub socket: Arc<Mutex<TcpStream>>,
  pub uuid: Uuid,
}

pub struct SlaveListener;

impl SlaveListener {
  pub async fn begin(
    config: ServerConfig, connections: Arc<Mutex<HashMap<Uuid, SenderPacket>>>,
    runtime: Arc<Runtime>, drop_handler: &Arc<AtomicBool>,
  ) -> Result<(), Error> { todo!("Implement slave server") }
}
