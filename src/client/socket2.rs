use hydrogen::{HydrogenSocket, Stream as HydrogenStream};
use simplelog::{debug, error, info, trace, warn};
use std::{
  cell::UnsafeCell,
  collections::HashMap,
  io::{Error, ErrorKind, Read, Write},
  net::{Shutdown, TcpStream},
  os::{
    fd::FromRawFd,
    unix::io::{AsRawFd, RawFd},
  },
  sync::Arc,
};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ServerConfig {
  pub separator: String,
  pub listen: u16,
  pub threads: usize,
  pub concurrency: usize,
}

pub struct Stream {
  inner: TcpStream,
}

impl Stream {
  pub fn from_tcp_stream(tcp_stream: TcpStream) -> Stream {
    tcp_stream.set_nonblocking(true).unwrap();
    Stream {
      inner: tcp_stream,
    }
  }
}

impl HydrogenStream for Stream {
  // This method is called when epoll reports data is available for reading.
  fn recv(&mut self) -> Result<Vec<Vec<u8>>, Error> {
    let mut msgs = Vec::<Vec<u8>>::new();

    // Our socket is set to non-blocking, we need to read until
    // there is an error or the system returns WouldBlock.
    // TcpStream offers no guarantee it will return in non-blocking mode.
    // Double check OS specifics on this when using.
    // https://doc.rust-lang.org/std/io/trait.Read.html#tymethod.read
    let mut total_read = Vec::<u8>::new();
    loop {
      let mut buf = [0u8; 4098];
      let read_result = self.inner.read(&mut buf);
      if read_result.is_err() {
        let err = read_result.unwrap_err();
        if err.kind() == ErrorKind::WouldBlock {
          break;
        }

        return Err(err);
      }

      let num_read = read_result.unwrap();
      total_read.extend_from_slice(&buf[0..num_read]);
    }

    // Multiple frames, or "msgs", could have been gathered here. Break up
    // your frames here and save remainer somewhere to come back to on the
    // next reads....
    //
    // Frame break out code goes here
    //

    msgs.push(total_read);

    return Ok(msgs);
  }

  // This method is called when a previous attempt to write has returned `ErrorKind::WouldBlock`
  // and epoll has reported that the socket is now writable.
  fn send(&mut self, buf: &[u8]) -> Result<(), Error> {
    self.inner.write_all(buf)
  }

  // This method is called when connection has been reported as reset by epoll, or when any
  // `std::io::Error` has been returned.
  fn shutdown(&mut self) -> Result<(), Error> {
    self.inner.shutdown(Shutdown::Both)
  }
}

impl AsRawFd for Stream {
  fn as_raw_fd(&self) -> RawFd {
    self.inner.as_raw_fd()
  }
}

// The following will be our server that handles all reported events
struct Server {
  connections: HashMap<RawFd, Uuid>,
  config: ServerConfig,
}

impl hydrogen::Handler for Server {
  fn on_server_created(&mut self, _: RawFd) {
    // Do any secific flag/option setting on the underlying listening fd.
    // This will be the fd that accepts all incoming connections.
    info!("Server created");
    info!(
      "Listening on: 0.0.0.0:{}",
      self.config.listen
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
    let uuid = Uuid::new_v4();
    self.connections.insert(fd, uuid);
    info!("New connection: {}", uuid);
    Arc::new(UnsafeCell::new(stream))
  }

  fn on_data_received(&mut self, socket: HydrogenSocket, buffer: Vec<u8>) {
    // Called when a complete, consumer defined, chunk of data has been read.
    debug!("Received data: {:x?}", buffer);
    let buf = buffer.clone();
    socket.send(buf.as_slice());
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
  }
}

pub fn main(config: &ServerConfig) {
  let config = config.to_owned();
  hydrogen::begin(
    Box::new(Server {
      connections: HashMap::new(),
      config: config.to_owned(),
    }),
    hydrogen::Config {
      addr: String::from("0.0.0.0"),
      port: config.listen,
      max_threads: config.threads,
      pre_allocated: config.concurrency,
    },
  );
}
