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

use proxy_router::functions::{Client, PacketType, Runtime};
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
) -> () {
  while !Arc::clone(&drop_handler).load(Ordering::Relaxed) {
    let address = format!(
      "{}:{}",
      config.redirect_to.address, config.redirect_to.port
    );
    info!("Connecting to {address}...");
    let stream = TcpStream::connect(address.clone());
    if stream.is_err() {
      warn!("Failed to connect to {address}: {}", stream.unwrap_err());
      sleep(Duration::from_secs(5)).await;
      continue;
    }
    let mut stream = stream.unwrap();
    stream.set_nonblocking(true).unwrap();
    stream.set_nodelay(true).unwrap();
    info!("Connected to {address}");
    stream
      .write_all(
        Client::build_auth_packet(
          &config.auth.to_owned(),
          /* &vec![3000, 4000, 5000], */ &config.targets.iter().map(|x| x.port).collect::<Vec<u16>>(),
          &config.separator,
        )
        .as_slice(),
      )
      .unwrap();
    info!("Client authenticated");

    let stream = Arc::new(Mutex::new(stream));
    let (sender, receiver) = mpsc::channel::<Vec<u8>>();
    let connections: Arc<Mutex<HashMap<Uuid, Arc<Connection>>>> =
      Arc::new(Mutex::new(HashMap::new()));

    let stream_ref_clone = Arc::clone(&stream);
    let separator = config.separator.to_owned();
    let config_clone = config.clone();
    let connections_clone = Arc::clone(&connections);
    let sender = Arc::new(Mutex::new(sender));
    let atomic = Arc::clone(&drop_handler);
    let from_server = tokio::task::spawn(async move {
      let connections = connections_clone;
      while !atomic.load(Ordering::Relaxed) {
        let mut stream = stream_ref_clone.lock().await;
        let mut buffer: Vec<u8> = Vec::new();
        let mut times_read = 0;
        let msg: Result<Option<Vec<u8>>, Error> = loop {
          let mut buf: [u8; 4098] = [0u8; 4098];
          match stream.read(&mut buf) {
            | Ok(bytes_read) => {
              if bytes_read == 0 {
                if times_read > 0 {
                  break Ok(Some(buffer));
                }
                break Ok(None);
              }
              trace!("Read {bytes_read} bytes.");
              times_read += 1;
              buffer.extend_from_slice(&buf[..bytes_read]);
            },
            | Err(err) => {
              if err.kind() == ErrorKind::WouldBlock {
                if times_read > 0 {
                  break Ok(Some(buffer));
                }
                break Ok(None);
              } else {
                break Err(err);
              }
            },
          }
        };

        match msg {
          | Ok(msg) => {
            if let Some(msg) = msg {
              let packet =
                Client::parse_packet(&msg, &separator.as_bytes().to_vec());
              match packet {
                | Ok(packet) => match packet {
                  | PacketType::Data(packet) => {
                    let mut connections_lock = connections.lock().await;
                    let connection = connections_lock.get(&packet.id);
                    if let Some(connection) = connection {
                      let sender = connection.sender.lock().await;
                      sender.send(packet.body).unwrap();
                    } else {
                      let address = &config_clone
                        .targets
                        .iter()
                        .find(|x| x.port == packet.port)
                        .unwrap()
                        .address;
                      let connection = create_connection(
                        &packet.id,
                        format!("{address}:{}", packet.port),
                        &config_clone.clone(),
                        Arc::clone(&sender),
                        Arc::clone(&atomic),
                      );
                      match connection {
                        | Ok(connection) => {
                          connection
                            .sender
                            .lock()
                            .await
                            .send(packet.body)
                            .unwrap();
                          connections_lock
                            .insert(packet.id, Arc::new(connection));
                          info!("{} open.", packet.id);
                        },
                        | Err(err) => {
                          error!("Error: {}", err);
                        },
                      }
                    }
                  },
                  | PacketType::Close(packet) => {
                    let mut connections_lock = connections.lock().await;
                    let connection = connections_lock.get(&packet.id);
                    if let Some(connection) = connection {
                      connection.drop_handler.store(true, Ordering::Relaxed);
                      connections_lock.remove(&packet.id);
                      info!("Closing {}.", packet.id);
                    }
                  },
                  | _ => {
                    warn!("Received invalid packet type, ignoring.");
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
            stream.shutdown(Shutdown::Both).ok();
            break;
          },
        }
      }
      atomic.store(true, Ordering::Relaxed);
    });

    let atomic = Arc::clone(&drop_handler);
    let stream_ref_clone = Arc::clone(&stream);
    let receiver = Arc::new(Mutex::new(receiver));
    let from_client = tokio::task::spawn(async move {
      while !atomic.load(Ordering::Relaxed) {
        let mut stream = stream_ref_clone.lock().await;
        let receiver = receiver.lock().await;
        let msg = receiver.try_recv();
        match msg {
          | Ok(msg) => {
            stream.write_all(msg.as_slice()).unwrap();
            trace!("Wrote {} to the main socket", msg.len());
          },
          | _ => {},
        }
      }
      atomic.store(true, Ordering::Relaxed);
    });

    from_server.await.unwrap();
    info!("Server side stopped.");
    from_client.await.unwrap();
    info!("Client side stopped.");
    let stream = stream.lock().await;
    stream.shutdown(Shutdown::Both).ok();
    info!("Main stream shutdown.");
  }
}

fn create_connection<Address>(
  uuid: &Uuid, address: Address, config: &Config<Runtime>,
  sender: Arc<Mutex<mpsc::Sender<Vec<u8>>>>, drop_handler: Arc<AtomicBool>,
) -> Result<Connection, Error>
where
  Address: ToSocketAddrs, std::string::String: From<Address>
{
  let address: String = address.into();
  let atomic = Arc::new(AtomicBool::new(false));
  
  // Channel to send data to the server
  let tx = Arc::clone(&sender);

  let (sender, rx) = mpsc::channel::<Vec<u8>>();

  // Channel to receive data from the server
  let rx = Arc::new(Mutex::new(rx));

  let stream = TcpStream::connect(address.to_owned())?;
  stream.set_nodelay(true)?;
  stream.set_nonblocking(true)?;
  let stream = Arc::new(Mutex::new(stream));
  debug!("{uuid} -> {address}");

  let atomic_clone = Arc::clone(&atomic);
  let stream_clone = Arc::clone(&stream);
  let uuid = uuid.to_owned();
  let config_clone = config.clone();
  tokio::task::spawn(async move {
    while !drop_handler.load(Ordering::Relaxed)
      && !atomic_clone.load(Ordering::Relaxed)
    {
      let tx = tx.lock().await;
      let rx = rx.lock().await;
      let mut stream = stream_clone.lock().await;
      let mut buffer: Vec<u8> = Vec::new();
      let mut times_read = 0;
      let msg: Result<Option<Vec<u8>>, Error> = loop {
        let mut buf: [u8; 4098] = [0u8; 4098];
        match stream.read(&mut buf) {
          | Ok(bytes_read) => {
            if bytes_read == 0 {
              if times_read > 0 {
                break Ok(Some(buffer));
              }
              break Ok(None);
            }
            times_read += 1;
            buffer.extend_from_slice(&buf[..bytes_read]);
          },
          | Err(err) => {
            if err.kind() == ErrorKind::WouldBlock {
              if times_read > 0 {
                break Ok(Some(buffer));
              }
              break Ok(None);
            } else {
              break Err(err);
            }
          },
        }
      };

      match msg {
        | Ok(msg) => {
          if let Some(msg) = msg {
            let packet =
              Client::build_data_packet(&uuid, &config_clone.separator, &msg);
            tx.send(packet).unwrap();
            trace!("{uuid} <- {address}");
          }
        },
        | Err(err) => {
          debug!("Error: {}", err);
          stream.shutdown(Shutdown::Both).ok();
          break;
        },
      }

      let message = rx.try_recv();
      match message {
        | Ok(message) => {
          stream.write_all(message.as_slice()).unwrap();
          trace!("{uuid} -> {address}");
        },
        | _ => {},
      }
    }
    stream_clone.lock().await.shutdown(Shutdown::Both).ok();
    debug!("{uuid} closed.");
  });

  Ok(Connection {
    sender: Arc::new(Mutex::new(sender)),
    drop_handler: atomic,
  })
}
