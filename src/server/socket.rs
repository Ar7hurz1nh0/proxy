use hydrogen::{HydrogenSocket, Stream as HydrogenStream};
use proxy_router::{
  constants::{Runtime, Stream},
  functions::{PacketType, Server, Warning},
};
use simplelog::{debug, error, info};
use std::{
  cell::UnsafeCell,
  collections::HashMap,
  io::Error,
  net::TcpStream,
  os::{
    fd::FromRawFd,
    unix::io::{AsRawFd, RawFd},
  },
  sync::{Arc, Mutex},
};
use uuid::Uuid;

use crate::slave::SenderPacket;

use super::slave::{Address, ServerConfig, SlaveListener};

// The following will be our server that handles all reported events
pub struct MasterListener {
  config: crate::config::Config<Runtime>,
  was_authed: bool,
  warn: Warning,
  connections: Arc<Mutex<HashMap<Uuid, SenderPacket>>>,
}

impl hydrogen::Handler for MasterListener {
  fn on_server_created(&mut self, _: RawFd) {
    // Do any secific flag/option setting on the underlying listening fd.
    // This will be the fd that accepts all incoming connections.
    info!("Server created");
    info!(
      "Listening on: {}:{}",
      self.config.listen.host, self.config.listen.port
    );
    info!("Max threads: {}", self.config.threads);
    info!(
      "Concurrency expected: {}",
      self.config.concurrency
    );
    info!("Waiting for authentication...");
  }

  fn on_new_connection(
    &mut self, fd: RawFd,
  ) -> Arc<UnsafeCell<dyn HydrogenStream>> {
    // With the passed fd, create your type that implements `hydrogen::Stream`
    // and return it.

    // For example:
    let tcp_stream = unsafe { TcpStream::from_raw_fd(fd) };
    let stream = Stream::from_tcp_stream(tcp_stream);
    info!("New connection: {fd}");
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
                info!(
                  "Authenticated connection: {}",
                  socket.as_raw_fd()
                );
                for port in packet.ports {
                  SlaveListener::begin(&ServerConfig {
                    separator: self.config.separator.clone(),
                    listen: Address {
                      port,
                      addr: self.config.listen.host.clone(),
                    },
                    threads: self.config.threads,
                    concurrency: self.config.concurrency,
                    socket: Arc::new(Mutex::new(socket.clone())),
                    connections: Arc::clone(&self.connections),
                  });
                }
              }
            },
            | _ => {
              error!("Expected a auth packet, got something else. Closing connection.");
              match socket.shutdown() {
                | Ok(_) => info!("Shutdown connection"),
                | Err(err) => error!("Error shutting down connection: {err}"),
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
    // Called when a connection has been removed from the watch list, with the
    // `std::io::Error` as the reason removed.
    debug!("{fd} removed: {err}");
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
      }),
      hydrogen::Config {
        addr: config.listen.host,
        port: config.listen.port,
        max_threads: config.threads,
        pre_allocated: config.concurrency,
      },
    );
  }
}
