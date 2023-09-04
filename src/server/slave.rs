use proxy::utils::Server;
use simplelog::{error, info, trace};
use std::{
  collections::HashMap,
  io::{Error, Read, Write},
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  thread,
  time::Duration,
};
use tokio::{runtime::Runtime, sync::Mutex, net::{TcpListener, TcpStream}, io::AsyncReadExt};
use uuid::Uuid;

use crate::slave;

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

pub struct SlaveListener {
  pub config: ServerConfig,
  pub connections: Arc<Mutex<HashMap<Uuid, SenderPacket>>>,
  pub runtime: Arc<Runtime>,
  pub drop_handler: Arc<AtomicBool>,
}

impl SlaveListener {
  pub async fn begin(
    config: ServerConfig, connections: Arc<Mutex<HashMap<Uuid, SenderPacket>>>,
    runtime: Arc<Runtime>, drop_handler: &Arc<AtomicBool>,
  ) -> Result<(), Error> { 
    let slave = SlaveListener {
      config: config.to_owned(),
      connections: Arc::clone(&connections),
      runtime: Arc::clone(&runtime),
      drop_handler: Arc::clone(&drop_handler),
    };
    let slave = Arc::new(slave);

    let listener = runtime.spawn(async move {
      let listener = TcpListener::bind((config.listen.addr.as_str(), config.listen.port)).await.unwrap();
      info!("Listening on {}:{}", config.listen.addr, config.listen.port);
      loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let uuid = Uuid::new_v4();
        slave.connections.lock().await.insert(uuid, SenderPacket {
          socket: Arc::new(Mutex::new(socket)),
          uuid: uuid.to_owned(),
        });
        let slave_clone = Arc::clone(&slave);
        Arc::clone(&slave).runtime.spawn(async move {
          slave_clone.on_new_connection(uuid).await.unwrap();
          loop {
            let mut buffer = vec![0; 1024];
            let slave = slave_clone.connections.lock().await;
            let socket = &slave.get(&uuid).unwrap().socket;
            let mut socket_clone = socket.lock().await;
            let bytes_read = socket_clone.read(&mut buffer).await.unwrap();
            if bytes_read == 0 {
              break;
            }
            slave_clone.on_data(Arc::clone(&socket), uuid, buffer[..bytes_read].to_vec()).await.unwrap();
          }
        });
      }
    });

    let drop_handler = Arc::clone(&drop_handler);
    runtime.spawn(async move {
      while !drop_handler.load(Ordering::Relaxed) {
        tokio::time::sleep(Duration::from_millis(1000)).await;
      }
      listener.abort();
    });

    Ok(())
  }

  pub async fn on_new_connection(
    &self, uuid: Uuid,
  ) -> Result<(), Error> { todo!("Implement slave server") }

  pub async fn on_data(
    &self, mut socket: Arc<Mutex<TcpStream>>, uuid: Uuid, data: Vec<u8>,
  ) -> Result<(), Error> { todo!("Implement slave server") }
}
