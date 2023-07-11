use hydrogen::{HydrogenSocket, Stream as HydrogenStream};
use proxy_router::functions::{PacketType, Runtime, Server, Stream, Warning};
use simplelog::{debug, error, info, warn};
use std::{
  cell::UnsafeCell,
  collections::HashMap,
  io::Error,
  net::TcpStream,
  os::{
    fd::FromRawFd,
    unix::io::{AsRawFd, RawFd},
  },
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
};
use uuid::Uuid;

use crate::slave::SenderPacket;

use super::slave::{Address, ServerConfig, SlaveListener};

// The following will be our server that handles all reported events
pub struct MasterListener {
  config: crate::config::Config<Runtime>,
  was_authed: bool,
  auth_fd: Option<RawFd>,
  warn: Warning,
  connections: Arc<Mutex<HashMap<Uuid, SenderPacket>>>,
  slaves: Vec<Arc<AtomicBool>>,
}

impl hydrogen::Handler for MasterListener {
  fn on_server_created(&mut self, _: RawFd) {
    // Do any secific flag/option setting on the underlying listening fd.
    // This will be the fd that accepts all incoming connections.
    info!("<green>Server created</>");
    info!(
      "<green>Listening on:</> <magenta>{}</>:<yellow>{}</>",
      self.config.listen.host, self.config.listen.port
    );
    debug!("Max threads: {}", self.config.threads);
    debug!(
      "Concurrency expected: {}",
      self.config.concurrency
    );
    info!("<yellow>Waiting for authentication...</>");
  }

  fn on_new_connection(
    &mut self, fd: RawFd,
  ) -> Arc<UnsafeCell<dyn HydrogenStream>> {
    let tcp_stream = unsafe { TcpStream::from_raw_fd(fd) };
    let stream = Stream::from_tcp_stream(tcp_stream);
    let uuid = stream.id;
    info!("New connection with fd {fd} has id {uuid}");
    Arc::new(UnsafeCell::new(stream))
  }

  fn on_data_received(&mut self, mut socket: HydrogenSocket, buffer: Vec<u8>) {
    // Called when a complete, consumer defined, chunk of data has been read.
    if !self.was_authed {
      let packet = Server::parse_packet(
        buffer,
        &self.config.separator.as_bytes().to_vec(),
      );
      match packet {
        | Ok(packet) => {
          match packet {
            | PacketType::Auth(packet) => {
              if self.config.auth.as_bytes().to_vec() == packet.body {
                self.was_authed = true;
                self.auth_fd = Some(socket.as_raw_fd());
                info!(
                  "Authenticated connection: {}",
                  socket.as_raw_fd()
                );
                for port in packet.ports {
                  let config = ServerConfig {
                    separator: self.config.separator.clone(),
                    listen: Address {
                      port,
                      addr: self.config.listen.host.clone(),
                    },
                    threads: self.config.threads,
                    concurrency: self.config.concurrency,
                    socket: Arc::new(Mutex::new(socket.clone())),
                    connections: Arc::clone(&self.connections),
                  };
                  let atomic = Arc::new(AtomicBool::new(false));
                  // FIXME: The next line somehow causes the hydrogen::Handler::on_connection_removed to not be called
                  //        when the connection is closed. This is a problem because the server should restart when
                  //        the main connection is closed.
                  match SlaveListener::begin(config, &atomic) {
                    | Ok(_) => {
                      info!("Started slave on port: {port}");
                      self.slaves.push(atomic);
                    },
                    | Err(err) => {
                      error!("Failed to start slave on port {port}: {err}");
                    },
                  }
                }
              }
            },
            | _ => {
              error!("Expected a auth packet, got something else. Closing connection.");
              match socket.shutdown() {
                | Ok(_) => info!("Shutdown connection"),
                | Err(err) => warn!("Error shutting down connection: {err}"),
              }
            },
          }
        },
        | Err(err) => {
          error!("Error parsing packet: {}", err.value());
          self.warn.warn(
            "This may result in a hanging connection or a broken pipe"
              .to_string(),
          );
        },
      }
    } else {
      let packet = Server::parse_packet(
        buffer,
        &self.config.separator.as_bytes().to_vec(),
      );
      match packet {
        | Ok(packet) => {
          match packet {
            | PacketType::Data(packet) => match self.connections.lock() {
              | Ok(connections) => match connections.get(&packet.id) {
                | Some(stream) => match stream.socket.lock() {
                  | Ok(mut socket) => match socket.send(&packet.body) {
                    | Ok(_) => debug!(
                      "Wrote data to socket: {}",
                      socket.as_raw_fd()
                    ),
                    | Err(err) => error!(
                      "Failed to write data to socket ({}): {err}",
                      socket.as_raw_fd()
                    ),
                  },
                  | Err(err) => {
                    error!("Failed to aquire lock for socket: {err}");
                    self.warn.warn("This may result in a hanging connection or a broken pipe".to_string());
                  },
                },
                | None => debug!(
                  "Failed to find connection for socket: {}",
                  packet.id
                ),
              },
              | Err(err) => {
                error!("Failed while aquiring lock for connections: {err}");
                self.warn.warn(
                  "This may result in a hanging connection or a broken pipe"
                    .to_string(),
                );
              },
            },
            | PacketType::Close(packet) => match self.connections.lock() {
              | Ok(connections) => match connections.get(&packet.id) {
                | Some(connection) => match connection.socket.lock() {
                  | Ok(mut socket) => match socket.shutdown() {
                    | Ok(_) => debug!(
                      "Closed connection: {}",
                      socket.as_raw_fd()
                    ),
                    | Err(err) => error!("Failed to close connection: {err}"),
                  },
                  | Err(err) => error!(
                    "Failed to find connection for socket ({}): {err}",
                    socket.as_raw_fd()
                  ),
                },
                | None => error!(
                  "Failed to find connection for socket: {}",
                  socket.as_raw_fd()
                ),
              },
              | Err(err) => {
                error!("Failed while aquiring lock for connections: {err}");
                self.warn.warn(
                  "This may result in a hanging connection or a broken pipe"
                    .to_string(),
                );
              },
            },
            | _ => {
              error!(
              "Expected a data packet, got something else. Closing connection. (fd: {})",
              socket.as_raw_fd()
            );
              match socket.shutdown() {
                | Ok(_) => {
                  info!("Shutdown connection");
                },
                | Err(err) => {
                  error!("Error shutting down connection: {err}");
                },
              }
            },
          }
        },
        | Err(err) => {
          error!("Error parsing packet: {}", err.value());
          self.warn.warn(
            "This may result in a hanging connection or a broken pipe"
              .to_string(),
          );
        },
      }
    }
  }

  fn on_connection_removed(&mut self, fd: RawFd, err: Error) {
    debug!("{fd} removed: {err}");
    match self.auth_fd {
      | Some(auth_fd) => {
        if auth_fd == fd {
          warn!("Auth connection closed. Restarting server.");
          for atomic in &self.slaves {
            atomic.store(true, Ordering::Relaxed);
          }
          self.was_authed = false;
          self.auth_fd = None;
        }
      },
      | None => {},
    }
  }
}

impl MasterListener {
  pub fn start(config: &crate::config::Config<Runtime>) {
    let config = config.to_owned();
    hydrogen::begin(
      Box::new(MasterListener {
        config: config.to_owned(),
        was_authed: false,
        warn: Warning::new(5),
        connections: Arc::new(Mutex::new(HashMap::new())),
        slaves: vec![],
        auth_fd: None,
      }),
      hydrogen::Config {
        addr: config.listen.host,
        port: config.listen.port,
        max_threads: config.threads,
        pre_allocated: config.concurrency,
      },
      None,
    );
  }
}
