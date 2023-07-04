// use uuid::Uuid;
use std::net::{Shutdown, TcpStream};
use std::{
  io,
  io::{Read, Write},
};

use proxy_router::constants::Runtime;
use proxy_router::functions::Client;

use crate::config::Config;

pub fn connect(config: &Config<Runtime>) -> () {
  // Connect to the TCP server
  let mut stream = TcpStream::connect(format!(
    "127.0.0.1:{}",
    config.redirect_to.port
  ))
  .unwrap();
  stream
    .write_all(
      Client::build_auth_packet(
        &config.auth.to_owned(),
        &vec![3000, 4000, 5000],
        &config.separator,
      )
      .as_slice(),
    )
    .unwrap();

  loop {
    // Read input from the user
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let message = input.trim();

    // Send the message to the server
    stream.write_all(message.as_bytes()).unwrap();

    // Check if the user wants to quit
    if message == "quit" {
      break;
    }

    // Read the server's response
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let received_data = &buffer[..bytes_read];

    // Process received data
    println!(
      "Received: {}",
      String::from_utf8_lossy(received_data)
    );
  }

  // Close the TCP connection
  stream.shutdown(Shutdown::Both).unwrap();
}
