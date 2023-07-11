use hydrogen::Stream as HydrogenStream;
use proxy_router::functions::{PacketType, Runtime, Server};
use simplelog::{debug, error, info, trace, warn};
use std::{
  collections::HashMap,
  io::Error,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  time::Duration,
};
use tokio::{sync::Mutex, time::sleep, net::TcpListener, io::AsyncReadExt};
use uuid::Uuid;

use crate::slave::SenderPacket;

use super::slave::{Address, ServerConfig, SlaveListener};

#[derive(Clone)]
pub struct MasterListener {
  config: crate::config::Config<Runtime>,
  was_authed: bool,
  connections: Arc<std::sync::Mutex<HashMap<Uuid, SenderPacket>>>,
}

impl MasterListener {
  pub fn start(
    config: &crate::config::Config<Runtime>, drop_handler: Arc<AtomicBool>,
  ) {
    let config = config.to_owned();
    server(
      MasterListener {
        config: config.to_owned(),
        was_authed: false,
        connections: Arc::new(std::sync::Mutex::new(HashMap::new())),
      },
      drop_handler,
    )
  }
}

#[tokio::main]
async fn server(init: MasterListener, drop_handler: Arc<AtomicBool>) {
  while !Arc::clone(&drop_handler).load(Ordering::Relaxed) {
    let address = format!(
      "{}:{}",
      init.config.listen.host, init.config.listen.port
    );
    let listener = TcpListener::bind(address.to_owned()).await;
    if listener.is_err() {
      error!(
        "Couldn't bind server to {address}: {}",
        listener.unwrap_err()
      );
      sleep(Duration::from_secs(5)).await;
      continue;
    }
    let listener = listener.unwrap();
    // listener.set_nonblocking(true).unwrap();
    let init = Arc::new(Mutex::new(init.clone()));
    info!("Successfully bind server to {address}");

    let listener = Arc::new(Mutex::new(listener));

    let atomic = Arc::clone(&drop_handler);
    let listener_clone = Arc::clone(&listener);
    let separator = init.lock().await.config.separator.to_owned();
    let main_loop = tokio::task::spawn(async move {
      while !atomic.load(Ordering::Relaxed) {
        let listener = listener_clone.lock().await;
        let stream = listener.accept().await;
        if stream.is_err() {
          continue;
        }
        let (stream, _) = stream.unwrap();
        let stream = Arc::new(Mutex::new(stream));
        let mut buffer: Vec<u8> = Vec::new();
        // let mut times_read = 0;

        let msg: Result<Option<Vec<u8>>, Error> = loop {
          let mut buf: [u8; 4098] = [0u8; 4098];
          let stream_lock = Arc::clone(&stream);
          let mut stream_lock = stream_lock.lock().await;
          // match stream_lock.read(&mut buf).await {
          //   | Ok(bytes_read) => {
          //     if bytes_read == 0 {
          //       if times_read > 0 {
          //         break Ok(Some(buffer));
          //       }
          //       break Ok(None);
          //     }
          //     trace!("Read {bytes_read} bytes.");
          //     times_read += 1;
          //     buffer.extend_from_slice(&buf[..bytes_read]);
          //   },
          //   | Err(err) => {
          //     if err.kind() == ErrorKind::WouldBlock {
          //       if times_read > 0 {
          //         break Ok(Some(buffer));
          //       }
          //       break Ok(None);
          //     } else {
          //       break Err(err);
          //     }
          //   },
          // }
          let bytes_read = stream_lock.read(&mut buf).await.unwrap();
          if bytes_read == 0 {
            break Ok(None)
          }
          buffer.extend_from_slice(&buf[..bytes_read]);
          break Ok(Some(buffer));
        };

        match msg {
          | Ok(msg) => {
            if let Some(msg) = msg {
              let packet =
                Server::parse_packet(&msg, &separator.as_bytes().to_vec());
              let mut init = init.lock().await;
              match packet {
                | Ok(packet) => match packet {
                  | PacketType::Data(packet) => {
                    if init.was_authed {
                      let connections = init.connections.lock().unwrap();
                      let connection = connections.get(&packet.id);
                      if let Some(connection) = connection {
                        let mut socket = connection.socket.lock().unwrap();
                        socket.send(&packet.body).unwrap();
                        trace!("CLIENT -> {}", packet.id);
                      } else {
                        warn!("Received DATA packet for unknown connection!");
                      }
                    } else {
                      warn!(
                        "Unexpected DATA packet before any authentication!"
                      );
                    }
                  },
                  | PacketType::Close(packet) => {
                    if init.was_authed {
                      let connections = init.connections.lock().unwrap();
                      let connection = connections.get(&packet.id);
                      if let Some(connection) = connection {
                        let mut socket = connection.socket.lock().unwrap();
                        socket.shutdown().unwrap();
                        info!("{} closed.", packet.id);
                      } else {
                        warn!("Received CLOSE packet for unknown connection!");
                      }
                    } else {
                      warn!(
                        "Unexpected CLOSE packet before any authentication!"
                      );
                    }
                  },
                  | PacketType::Auth(packet) => {
                    if !init.was_authed {
                      if packet.body == init.config.auth.as_bytes().to_vec() {
                        info!("Client authenticated!");
                        init.was_authed = true;
                        for port in packet.ports {
                          let config = ServerConfig {
                            separator: init.config.separator.clone(),
                            listen: Address {
                              port,
                              addr: init.config.listen.host.clone(),
                            },
                            threads: init.config.threads,
                            concurrency: init.config.concurrency,
                            socket: Arc::clone(&stream),
                            connections: Arc::clone(&init.connections),
                          };
                          let atomic = Arc::clone(&atomic);
                          match SlaveListener::begin(config, &atomic) {
                            | Ok(_) => {
                              info!("Started slave on port: {port}");
                            },
                            | Err(err) => {
                              error!(
                                "Failed to start slave on port {port}: {err}"
                              );
                            },
                          }
                        }
                      }
                    } else {
                      warn!("Client tried to auth twice!")
                    }
                  },
                },
                | Err(err) => {
                  debug!("Msg lenght: {}", msg.len());
                  error!("Error: {}", err);
                },
              }
            }
          },
          | Err(err) => {
            debug!("Error: {}", err);
            break;
          },
        }
      }
      atomic.store(true, Ordering::Relaxed)
    });

    let atomic = Arc::clone(&drop_handler);
    while !atomic.load(Ordering::Relaxed) {}
    main_loop.abort();

    info!("Client connection ended.");
  }
}
