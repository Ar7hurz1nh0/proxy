use hydrogen::{HydrogenSocket, Stream as HydrogenStream};
use proxy_router::functions::{Server, Stream, Warning};
use simplelog::{debug, error, info};
use std::{
  cell::UnsafeCell,
  collections::HashMap,
  io::Error,
  net::TcpStream,
  os::{fd::FromRawFd, unix::io::RawFd},
  sync::{atomic::AtomicBool, Arc, Mutex}, thread::Builder,
};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Address {
  pub port: u16,
  pub addr: String,
}

#[derive(Clone)]
pub struct ServerConfig {
  pub separator: String,
  pub listen: Address,
  pub threads: usize,
  pub concurrency: usize,
  pub socket: Arc<Mutex<HydrogenSocket>>,
  pub connections: Arc<Mutex<HashMap<Uuid, SenderPacket>>>,
}

pub struct SenderPacket {
  pub socket: Arc<Mutex<Stream>>,
  pub fd: RawFd,
  pub uuid: Uuid,
}

// The following will be our server that handles all reported events
pub struct SlaveListener {
  connections: HashMap<RawFd, Uuid>,
  config: ServerConfig,
  socket: Arc<Mutex<HydrogenSocket>>,
  warn: Warning,
}

impl hydrogen::Handler for SlaveListener {
  fn on_server_created(&mut self, _: RawFd) {
    // Do any secific flag/option setting on the underlying listening fd.
    // This will be the fd that accepts all incoming connections.
    info!("<blue>Slave server created</>");
    info!(
      "<blue>Listening on:</> <magenta>{}</>:<yellow>{}</>",
      self.config.listen.addr, self.config.listen.port
    );
    debug!("Max threads: {}", self.config.threads);
    debug!(
      "Concurrency expected: {}",
      self.config.concurrency
    );
  }

  fn on_new_connection(
    &mut self, fd: RawFd,
  ) -> Arc<UnsafeCell<dyn HydrogenStream>> {
    // With the passed fd, create your type that implements `hydrogen::Stream`
    // and return it.

    // For example:
    let tcp_stream = unsafe { TcpStream::from_raw_fd(fd) };
    let stream = Stream::from_tcp_stream(tcp_stream);
    self.connections.insert(fd, stream.id);
    info!("New connection: {}", stream.id);
    match self.config.connections.lock() {
      | Ok(mut connections) => {
        connections.insert(
          stream.id.to_owned(),
          SenderPacket {
            socket: Arc::new(Mutex::new(stream.to_owned())),
            fd: fd.to_owned(),
            uuid: stream.id.to_owned(),
          },
        );
      },
      | Err(err) => {
        error!("Failed while aquiring lock from connections: {err}");
        self.warn.warn(
          "This may result in a hanging connection or a broken pipe"
            .to_string(),
        );
      },
    }
    Arc::new(UnsafeCell::new(stream))
  }

  fn on_data_received(&mut self, socket: HydrogenSocket, buffer: Vec<u8>) {
    // Called when a complete, consumer defined, chunk of data has been read.
    match self.connections.get(&socket.arc_connection.fd) {
      | Some(id) => {
        debug!("Received data from {id}");
        let packet = Server::build_data_packet(
          &id.to_owned(),
          &self.config.listen.port,
          &self.config.separator,
          &buffer,
        );
        match self.socket.lock() {
          | Ok(master_socket) => {
            master_socket.send(packet.as_slice());
          },
          | Err(err) => {
            error!("Failed while aquiring lock from socket: {err}");
            self.warn.warn(
              "This may result in a hanging connection or a broken pipe"
                .to_string(),
            );
          },
        }
      },
      | None => {
        error!(
          "Unknown connection: {}",
          socket.arc_connection.fd
        );
        self.warn.warn(
          "This may result in a hanging connection or a broken pipe"
            .to_string(),
        );
      },
    }
  }

  fn on_connection_removed(&mut self, fd: RawFd, err: Error) {
    // Called when a connection has been removed from the watch list, with the
    // `std::io::Error` as the reason removed.
    match self.connections.get(&fd) {
      | Some(uuid) => {
        info!("{uuid} removed: {err}");
        self.connections.remove(&fd);
      },
      | None => {
        info!("Unknown connection removed: {}", err);
      },
    }
    match self.config.connections.lock() {
      | Ok(mut connections) => {
        connections.retain(|_, v| v.fd != fd);
      },
      | Err(err) => {
        error!("Failed while aquiring lock from connections: {err}");
        self.warn.warn(
          "This may result in a hanging connection or a broken pipe"
            .to_string(),
        );
      },
    }
  }
}

impl SlaveListener {
  pub fn begin(config: ServerConfig, drop_handler: &Arc<AtomicBool>) -> Result<std::thread::JoinHandle<()>, Error> {
    let atomic_clone = Arc::clone(&drop_handler);
    Builder::new()
      .name(format!("slave-{}", config.listen.port))
      .spawn(move || hydrogen::begin(
      Box::new(SlaveListener {
        connections: HashMap::new(),
        config: config.to_owned(),
        socket: Arc::clone(&config.socket),
        warn: Warning::new(5),
      }),
      hydrogen::Config {
        addr: config.listen.addr,
        port: config.listen.port,
        max_threads: config.threads,
        pre_allocated: config.concurrency,
      },
      Some(atomic_clone),
    ))
  }
}
