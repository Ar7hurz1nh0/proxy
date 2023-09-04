use std::{
  fmt::{Display, Formatter},
  string::FromUtf8Error,
};

use digest::Digest;
use rand::{distributions::Alphanumeric, Rng};
use sha1::Sha1;
use sha2::Sha512;
use uuid::Uuid;

pub enum PacketAction {
  /// Data packet
  ///
  /// This packet is used to send data to the server or client.
  ///
  /// # Usage
  ///
  /// The packet must follow this format:
  ///
  /// {action} {id} {port} {sha1} {sha512}{separator}{body}
  ///
  /// ## Example
  ///
  /// DATA 123e4567-e89b-12d3-a456-426614174000 8080 0a0a9f2a6772942557ab5355d76af442f8f65e01 374d794a95cdcfd8b35993185fef9ba368f160d8daf432d08ba9f1ed1e5abe6cc69291e0fa2fe0006a52570ef18c19def4e617c33ce52ef0a6e5fbe318cb0387\u0000Hello, world!
  DATA,

  /// Close packet
  ///
  /// This packet is used to signify that the connection should be closed.
  ///
  /// # Usage
  ///
  /// The packet must follow this format:
  ///
  /// {action} {id}{separator}
  ///
  /// ## Example
  ///
  /// CLOSE 123e4567-e89b-12d3-a456-426614174000\u0000
  CLOSE,

  /// Auth packet
  ///
  /// This packet is used to authenticate the first connection.
  ///
  /// # Usage
  ///
  /// The packet must follow this format:
  ///
  /// {action} {ports}{separator}{auth}
  ///
  /// ## Example
  ///
  /// AUTH 8080,8081,8082\u0000CH4ng3M3!
  AUTH,

  /// Auth try packet
  ///
  /// This packet is used to confirm that the auth packet was received, and respond with the auth status.
  ///
  /// # Usage
  ///
  /// The packet must follow this format:
  ///
  /// {action}{separator}{status}
  ///
  /// Where status is either "success" or "forbiden".
  ///
  /// ## Example
  ///
  /// AUTHTRY\u0000success
  AUTHTRY,

  /// Heartbeat packet
  ///
  /// This packet is used detect if the connection is still alive.
  /// The receiver end must respond as soon as possible with a heartbeat with the same nonce.
  /// If the receiver end does not respond in time, the connection is closed.
  ///
  /// # Usage
  ///
  /// The packet must follow this format:
  ///
  /// HEARTBEAT{separator}{nonce}
  ///
  /// ## Example
  ///
  /// HEARTBEAT\u0000a1b2c3d4e5f6
  HEARTBEAT,
}

#[derive(Debug, PartialEq)]
pub enum ParseErrorType {
  Type,
  Action,
  ID,
  Hash,
  Port,
  Ports,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
  Header(ParseErrorType),
  Other(ParseErrorType),
}

impl ParseErrorType {
  pub fn value(&self) -> String {
    match self {
      | ParseErrorType::Type => "Invalid type".to_string(),
      | ParseErrorType::Action => "Invalid action".to_string(),
      | ParseErrorType::ID => "Invalid ID".to_string(),
      | ParseErrorType::Hash => "Invalid hash".to_string(),
      | ParseErrorType::Port => "Invalid port".to_string(),
      | ParseErrorType::Ports => "Invalid ports".to_string(),
    }
  }
}

impl Display for ParseErrorType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.value())
  }
}

impl ParseError {
  pub fn value(&self) -> String {
    match self {
      | ParseError::Header(error) => {
        format!("Invalid header: {}", error.value())
      },
      | ParseError::Other(error) => {
        format!("Invalid packet: {}", error.value())
      },
    }
  }
}

impl Display for ParseError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.value())
  }
}

impl PacketAction {
  pub fn from_string(string: String) -> PacketAction {
    match string.to_lowercase().as_str() {
      | "data" => PacketAction::DATA,
      | "close" => PacketAction::CLOSE,
      | "auth" => PacketAction::AUTH,
      | "authtry" => PacketAction::AUTHTRY,
      | "heartbeat" => PacketAction::HEARTBEAT,
      | _ => panic!("Invalid packet type: {}", string),
    }
  }

  pub fn value(&self) -> String {
    match self {
      | PacketAction::DATA => "DATA".to_string(),
      | PacketAction::CLOSE => "CLOSE".to_string(),
      | PacketAction::AUTH => "AUTH".to_string(),
      | PacketAction::AUTHTRY => "AUTHTRY".to_string(),
      | PacketAction::HEARTBEAT => "HEARTBEAT".to_string(),
    }
  }
}

pub struct Server;
pub struct Client;
pub struct Data;
pub struct Auth;
pub struct Close;
pub struct AuthTry;
pub struct Heartbeat;

pub trait Environment {}
impl Environment for Server {}
impl Environment for Client {}

pub trait PacketTrait<Env: Environment> {
  type ID;
  type PORT;
  type PORTS;
  type SHA1;
  type SHA512;
}

impl PacketTrait<Client> for Data {
  type SHA1 = String;
  type SHA512 = String;
  type PORTS = ();
  type ID = Uuid;
  type PORT = ();
}

impl PacketTrait<Client> for Auth {
  type SHA1 = ();
  type SHA512 = ();
  type PORTS = Vec<u16>;
  type ID = ();
  type PORT = ();
}

impl PacketTrait<Client> for Close {
  type SHA1 = ();
  type SHA512 = ();
  type PORTS = ();
  type ID = Uuid;
  type PORT = ();
}

impl PacketTrait<Client> for AuthTry {
  type SHA1 = ();
  type SHA512 = ();
  type PORTS = ();
  type ID = ();
  type PORT = ();
}

impl PacketTrait<Client> for Heartbeat {
  type SHA1 = ();
  type SHA512 = ();
  type PORTS = ();
  type ID = ();
  type PORT = ();
}

impl PacketTrait<Server> for Data {
  type SHA1 = String;
  type SHA512 = String;
  type PORTS = ();
  type ID = Uuid;
  type PORT = u16;
}

impl PacketTrait<Server> for Auth {
  type SHA1 = ();
  type SHA512 = ();
  type PORTS = ();
  type ID = ();
  type PORT = ();
}

impl PacketTrait<Server> for Close {
  type SHA1 = ();
  type SHA512 = ();
  type PORTS = ();
  type ID = Uuid;
  type PORT = ();
}

impl PacketTrait<Server> for AuthTry {
  type SHA1 = ();
  type SHA512 = ();
  type PORTS = ();
  type ID = ();
  type PORT = ();
}

impl PacketTrait<Server> for Heartbeat {
  type SHA1 = ();
  type SHA512 = ();
  type PORTS = ();
  type ID = ();
  type PORT = ();
}

pub struct Packet<Env: Environment, PacketSubset>
where
  PacketSubset: PacketTrait<Env>,
{
  pub action: PacketAction,
  pub id: PacketSubset::ID,
  pub port: PacketSubset::PORT,
  pub ports: PacketSubset::PORTS,
  pub sha1: PacketSubset::SHA1,
  pub sha512: PacketSubset::SHA512,
  pub body: Vec<u8>,
}

pub enum PacketType<Env: Environment>
where
  Data: PacketTrait<Env>,
  Auth: PacketTrait<Env>,
  Close: PacketTrait<Env>,
  AuthTry: PacketTrait<Env>,
  Heartbeat: PacketTrait<Env>,
{
  Data(Packet<Env, Data>),
  Auth(Packet<Env, Auth>),
  Close(Packet<Env, Close>),
  AuthTry(Packet<Env, AuthTry>),
  Heartbeat(Packet<Env, Heartbeat>),
}

pub fn hash_sha1(data: &Vec<u8>) -> String {
  let mut sha1 = Sha1::new();
  sha1.update(data);
  let result_sha1 = sha1.finalize();
  format!("{:x}", result_sha1)
}

pub fn hash_sha512(data: &Vec<u8>) -> String {
  let mut sha512 = Sha512::new();
  sha512.update(data);
  let result_sha512 = sha512.finalize();
  format!("{:x}", result_sha512)
}

pub fn split(
  packet: &Vec<u8>, separator: &Vec<u8>,
) -> Option<(Vec<u8>, Vec<u8>)> {
  if separator.is_empty() || packet.is_empty() {
    return None;
  }

  let mut first_part = Vec::new();
  let mut cache: Vec<u8> = Vec::new();
  let mut separator_index = 0;
  let mut i: usize = 0;

  for byte in packet.clone() {
    i += 1;
    if byte == separator[separator_index] {
      separator_index += 1;
      cache.push(byte);

      if separator_index >= separator.len() {
        let second_part = packet.split_at(i).1.to_vec();
        return Some((first_part, second_part));
      }
    } else {
      if separator_index > 0 {
        first_part.extend(cache.clone());
        cache.clear();
      }
      first_part.push(byte);
      separator_index = 0;
    }
  }

  None
}

impl Server {
  pub fn build_data_packet(
    id: &Uuid, port: &u16, separator: &Vec<u8>, data: &Vec<u8>,
  ) -> Result<Vec<u8>, FromUtf8Error> {
    let separator = String::from_utf8(separator.to_owned())?;
    let id = id.to_string();
    let packet = format!(
      "{} {id} {port} {} {}{separator}",
      PacketAction::DATA.value(),
      hash_sha1(&data),
      hash_sha512(&data),
    );
    let mut packet = packet.as_bytes().to_vec();
    packet.extend(data);
    Ok(packet)
  }

  pub fn build_close_packet(
    id: &Uuid, separator: &Vec<u8>,
  ) -> Result<Vec<u8>, FromUtf8Error> {
    let separator = String::from_utf8(separator.to_owned())?;
    let id = id.to_string();
    let packet = format!(
      "{} {id}{separator}",
      PacketAction::CLOSE.value()
    );
    Ok(packet.into_bytes())
  }

  pub fn build_authtry_packet(
    separator: &Vec<u8>, success: &bool,
  ) -> Result<Vec<u8>, FromUtf8Error> {
    let separator = String::from_utf8(separator.to_owned())?;
    let success = if *success {
      "success"
    } else {
      "forbidden"
    };
    let packet = format!(
      "{}{separator}{success}",
      PacketAction::AUTHTRY.value()
    );
    Ok(packet.into_bytes())
  }

  pub fn build_heartbeat_packet(
    separator: &Vec<u8>, nonce: &String,
  ) -> Result<Vec<u8>, FromUtf8Error> {
    let separator = String::from_utf8(separator.to_owned())?;
    let packet = format!(
      "{}{separator}{nonce}",
      PacketAction::HEARTBEAT.value()
    );
    Ok(packet.into_bytes())
  }

  pub fn gen_nonce() -> String {
    rand::thread_rng()
      .sample_iter(&Alphanumeric)
      .take(32)
      .map(char::from)
      .collect()
  }

  ///
  /// Parses a packet from the client
  ///
  pub fn parse_packet(
    packet: &Vec<u8>, separator: &Vec<u8>,
  ) -> Result<PacketType<Client>, ParseError> {
    let (header, body) = split(&packet, separator)
      .ok_or(ParseError::Header(ParseErrorType::Type))?;
    let (action, p) = split(&header, &" ".as_bytes().to_vec()).ok_or(
      ParseError::Header(ParseErrorType::Action),
    )?;

    let action =
      PacketAction::from_string(String::from_utf8(action).ok().ok_or(
        ParseError::Other(ParseErrorType::Action),
      )?);

    match &action {
      | PacketAction::DATA => {
        let (id, p) = split(&p, &" ".as_bytes().to_vec())
          .ok_or(ParseError::Header(ParseErrorType::ID))?;
        let id = String::from_utf8(id)
          .ok()
          .ok_or(ParseError::Other(ParseErrorType::ID))?;

        let id = Uuid::parse_str(&id)
          .ok()
          .ok_or(ParseError::Other(ParseErrorType::ID))?;
        let (sha1, sha512) = split(&p, &" ".as_bytes().to_vec())
          .ok_or(ParseError::Header(ParseErrorType::Hash))?;
        let sha1 = String::from_utf8(sha1)
          .ok()
          .ok_or(ParseError::Other(ParseErrorType::Hash))?;
        let sha512 = String::from_utf8(sha512)
          .ok()
          .ok_or(ParseError::Other(ParseErrorType::Hash))?;
        Ok(PacketType::Data(Packet {
          action,
          id,
          port: (),
          ports: (),
          sha1,
          sha512,
          body,
        }))
      },
      | PacketAction::AUTH => {
        let ports = p;
        let ports = String::from_utf8(ports)
          .ok()
          .ok_or(ParseError::Other(ParseErrorType::Ports))?;
        let ports = ports
          .split(",")
          .map(|port| {
            port
              .parse::<u16>()
              .ok()
              .ok_or(ParseError::Other(ParseErrorType::Ports))
          })
          .collect::<Result<Vec<u16>, ParseError>>()?;
        Ok(PacketType::Auth(Packet {
          action,
          id: (),
          port: (),
          ports,
          sha1: (),
          sha512: (),
          body,
        }))
      },
      | PacketAction::CLOSE => {
        let id = String::from_utf8(p)
          .ok()
          .ok_or(ParseError::Other(ParseErrorType::ID))?;
        let id = Uuid::parse_str(&id)
          .ok()
          .ok_or(ParseError::Other(ParseErrorType::ID))?;
        Ok(PacketType::Close(Packet {
          action,
          id,
          port: (),
          ports: (),
          sha1: (),
          sha512: (),
          body,
        }))
      },
      | PacketAction::AUTHTRY => Err(ParseError::Other(ParseErrorType::Type)),
      | PacketAction::HEARTBEAT => Ok(PacketType::Heartbeat(Packet {
        action,
        id: (),
        port: (),
        ports: (),
        sha1: (),
        sha512: (),
        body,
      })),
    }
  }
}

impl Client {
  pub fn build_data_packet(
    id: &Uuid, separator: &Vec<u8>, data: &Vec<u8>,
  ) -> Result<Vec<u8>, FromUtf8Error> {
    let separator = String::from_utf8(separator.to_owned())?;
    let id = id.to_string();
    let packet = format!(
      "{} {id} {} {}{separator}",
      PacketAction::DATA.value(),
      hash_sha1(&data),
      hash_sha512(&data),
    );
    let mut packet = packet.as_bytes().to_vec();
    packet.extend(data);
    Ok(packet)
  }

  pub fn build_close_packet(
    id: &Uuid, separator: &Vec<u8>,
  ) -> Result<Vec<u8>, FromUtf8Error> {
    let separator = String::from_utf8(separator.to_owned())?;
    let id = id.to_string();
    let packet = format!(
      "{} {id}{separator}",
      PacketAction::CLOSE.value()
    );
    Ok(packet.into_bytes())
  }

  pub fn build_auth_packet(
    auth: &Vec<u8>, ports: &Vec<u16>, separator: &Vec<u8>,
  ) -> Result<Vec<u8>, FromUtf8Error> {
    let auth = String::from_utf8(auth.to_owned())?;
    let separator = String::from_utf8(separator.to_owned())?;
    let ports_string = ports
      .iter()
      .map(|port| port.to_string())
      .collect::<Vec<String>>()
      .join(",");
    let packet = format!(
      "{} {ports_string}{separator}{auth}",
      PacketAction::AUTH.value()
    );
    Ok(packet.into_bytes())
  }

  pub fn build_heartbeat_packet(
    separator: &Vec<u8>, nonce: &Vec<u8>,
  ) -> Result<Vec<u8>, FromUtf8Error> {
    let separator = String::from_utf8(separator.to_owned())?;
    let mut packet = format!(
      "{}{separator}",
      PacketAction::HEARTBEAT.value()
    )
    .into_bytes();
    packet.extend(nonce);
    Ok(packet)
  }

  ///
  /// Parses a packet from the server
  ///
  pub fn parse_packet(
    packet: &Vec<u8>, separator: &Vec<u8>,
  ) -> Result<PacketType<Server>, ParseError> {
    let (header, body) = split(&packet, separator)
      .ok_or(ParseError::Header(ParseErrorType::Type))?;

    let action = split(&header, &" ".as_bytes().to_vec());

    let (action, p) = if action.is_none() {
      let action =
        PacketAction::from_string(String::from_utf8(header).ok().ok_or(
          ParseError::Other(ParseErrorType::Action),
        )?);
      (action, vec![])
    } else {
      let (action, p) = action.unwrap();
      let action =
        PacketAction::from_string(String::from_utf8(action).ok().ok_or(
          ParseError::Other(ParseErrorType::Action),
        )?);
      (action, p)
    };

    match &action {
      | PacketAction::DATA => {
        let (id, p) = split(&p, &" ".as_bytes().to_vec())
          .ok_or(ParseError::Header(ParseErrorType::ID))?;
        let id = String::from_utf8(id)
          .ok()
          .ok_or(ParseError::Other(ParseErrorType::ID))?;
        let id = Uuid::parse_str(&id)
          .ok()
          .ok_or(ParseError::Other(ParseErrorType::ID))?;
        let (port, p) = split(&p, &" ".as_bytes().to_vec())
          .ok_or(ParseError::Header(ParseErrorType::Port))?;
        let port = String::from_utf8(port)
          .ok()
          .ok_or(ParseError::Other(ParseErrorType::Port))?
          .parse::<u16>()
          .ok()
          .ok_or(ParseError::Other(ParseErrorType::Port))?;
        let (sha1, sha512) = split(&p, &" ".as_bytes().to_vec())
          .ok_or(ParseError::Header(ParseErrorType::Hash))?;
        let sha1 = String::from_utf8(sha1)
          .ok()
          .ok_or(ParseError::Other(ParseErrorType::Hash))?;
        let sha512 = String::from_utf8(sha512)
          .ok()
          .ok_or(ParseError::Other(ParseErrorType::Hash))?;
        Ok(PacketType::Data(Packet {
          action,
          id,
          port,
          ports: (),
          sha1,
          sha512,
          body,
        }))
      },
      | PacketAction::CLOSE => {
        let id = String::from_utf8(p)
          .ok()
          .ok_or(ParseError::Other(ParseErrorType::ID))?;
        let id = Uuid::parse_str(&id)
          .ok()
          .ok_or(ParseError::Other(ParseErrorType::ID))?;
        Ok(PacketType::Close(Packet {
          action,
          id,
          port: (),
          ports: (),
          sha1: (),
          sha512: (),
          body,
        }))
      },
      | PacketAction::AUTH => Err(ParseError::Other(ParseErrorType::Type)),
      | PacketAction::AUTHTRY => Ok(PacketType::AuthTry(Packet {
        action,
        id: (),
        port: (),
        ports: (),
        sha1: (),
        sha512: (),
        body,
      })),
      | PacketAction::HEARTBEAT => Ok(PacketType::Heartbeat(Packet {
        action,
        id: (),
        port: (),
        ports: (),
        sha1: (),
        sha512: (),
        body,
      })),
    }
  }
}

#[derive(Clone, Debug)]
pub enum Runtime {}

#[derive(Clone, Debug)]
pub enum ConfigFile {}
