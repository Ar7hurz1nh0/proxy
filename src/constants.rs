use hydrogen::Stream as HydrogenStream;
use std::{
  io::{Error, ErrorKind, Read, Write},
  net::{Shutdown, TcpStream},
  os::unix::io::{AsRawFd, RawFd},
};
use uuid::Uuid;

pub const SETTING_FILE_PATH: &'static str = "config.json";

pub const LOG_PATH: &'static str = "logs";

pub const LOG_FILE: &'static str = "latest.log";

pub const BACKLOG: u16 = 100;

pub const DEFAULT_THREAD_COUNT: usize = 4;

#[derive(Clone, Debug)]
pub enum Runtime {}

#[derive(Clone, Debug)]
pub enum ConfigFile {}

pub struct Stream {
  inner: TcpStream,
  pub id: Uuid,
}

impl Stream {
  pub fn from_tcp_stream(tcp_stream: TcpStream) -> Stream {
    tcp_stream.set_nonblocking(true).unwrap();
    Stream {
      inner: tcp_stream,
      id: Uuid::new_v4(),
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

impl Clone for Stream {
  fn clone(&self) -> Self {
    Stream {
      inner: self.inner.try_clone().unwrap(),
      id: self.id,
    }
  }
}
