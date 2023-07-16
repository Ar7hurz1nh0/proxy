use std::{
  collections::HashMap,
  io::{Error, ErrorKind, Read, Write},
  net::{Shutdown, TcpStream, ToSocketAddrs},
  sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
  },
  time::Duration,
};

use proxy::utils::{Client, PacketType, Runtime};
use simplelog::{debug, error, info, trace, warn};
use tokio::{sync::Mutex, time::sleep};
use uuid::Uuid;

use crate::config::Config;

struct Connection {
  sender: Arc<Mutex<mpsc::Sender<Vec<u8>>>>,
  drop_handler: Arc<AtomicBool>,
}

#[tokio::main]
pub async fn connect(
  config: &Config<Runtime>, drop_handler: Arc<AtomicBool>,
) -> () { todo!("Implement client code") }