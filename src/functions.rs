use std::fmt::{Display, Formatter};

use digest::Digest;
use sha1::Sha1;
use sha2::Sha512;
use simplelog::warn;
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
}

#[derive(Debug)]
pub enum ParseErrorType {
  Type,
  Action,
  ID,
  Hash,
  Port,
  Ports,
}

#[derive(Debug)]
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
      | _ => panic!("Invalid packet type: {}", string),
    }
  }

  pub fn value(&self) -> String {
    match self {
      | PacketAction::DATA => "DATA".to_string(),
      | PacketAction::CLOSE => "CLOSE".to_string(),
      | PacketAction::AUTH => "AUTH".to_string(),
    }
  }
}

pub enum Server {}
pub enum Client {}
pub enum Data {}
pub enum Auth {}
pub enum Close {}

pub trait Environment {
  type PortType;
}

impl Environment for Server {
  type PortType = u16;
}

impl Environment for Client {
  type PortType = ();
}

pub trait PacketTrait {
  type Sha1Type;
  type Sha512Type;
  type PortsType;
  type IDType;
}

impl PacketTrait for Data {
  type Sha1Type = String;
  type Sha512Type = String;
  type PortsType = ();
  type IDType = Uuid;
}

impl PacketTrait for Auth {
  type Sha1Type = ();
  type Sha512Type = ();
  type PortsType = Vec<u16>;
  type IDType = ();
}

impl PacketTrait for Close {
  type Sha1Type = ();
  type Sha512Type = ();
  type PortsType = ();
  type IDType = Uuid;
}

pub struct Packet<Env: Environment, PacketSubset: PacketTrait> {
  pub action: PacketAction,
  pub id: PacketSubset::IDType,
  pub port: Env::PortType,
  pub ports: PacketSubset::PortsType,
  pub sha1: PacketSubset::Sha1Type,
  pub sha512: PacketSubset::Sha512Type,
  pub body: Vec<u8>,
}

pub enum PacketType<Env: Environment> {
  Data(Packet<Env, Data>),
  Auth(Packet<Env, Auth>),
  Close(Packet<Env, Close>),
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
    id: &Uuid, port: &u16, separator: &str, data: &Vec<u8>,
  ) -> Vec<u8> {
    let id = id.to_string();
    let packet = format!(
      "{} {id} {port} {} {}{separator}",
      PacketAction::DATA.value(),
      hash_sha1(&data),
      hash_sha512(&data),
    );
    let mut packet = packet.as_bytes().to_vec();
    packet.extend(data);
    packet
  }

  pub fn close_connection_packet(id: &Uuid, separator: &String) -> Vec<u8> {
    let id = id.to_string();
    let packet = format!(
      "{} {id}{separator}",
      PacketAction::CLOSE.value()
    );
    packet.as_bytes().to_vec()
  }

  ///
  /// Parses a packet from the client
  ///
  pub fn parse_packet(
    packet: Vec<u8>, separator: &Vec<u8>,
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
        let id: Uuid = Uuid::try_parse_ascii(p.as_slice())
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
    }
  }
}

impl Client {
  pub fn build_data_packet(
    id: &Uuid, separator: &str, data: &Vec<u8>,
  ) -> Vec<u8> {
    let id = id.to_string();
    let packet = format!(
      "{} {id} {} {}{separator}",
      PacketAction::DATA.value(),
      hash_sha1(&data),
      hash_sha512(&data),
    );
    let mut packet = packet.as_bytes().to_vec();
    packet.extend(data);
    packet
  }

  pub fn close_connection_packet(id: &Uuid, separator: &String) -> Vec<u8> {
    let id = id.to_string();
    let packet = format!(
      "{} {id} 0{separator}",
      PacketAction::CLOSE.value()
    );
    packet.as_bytes().to_vec()
  }

  pub fn build_auth_packet(
    auth: &String, ports: &Vec<u16>, separator: &String,
  ) -> Vec<u8> {
    let ports_string = ports
      .iter()
      .map(|port| port.to_string())
      .collect::<Vec<String>>()
      .join(",");
    let packet = format!(
      "{} {ports_string}{separator}{auth}",
      PacketAction::AUTH.value()
    );
    packet.as_bytes().to_vec()
  }

  ///
  /// Parses a packet from the server
  ///
  pub fn parse_packet(
    packet: Vec<u8>, separator: &Vec<u8>,
  ) -> Result<PacketType<Server>, ParseError> {
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
          port: 0,
          ports: (),
          sha1: (),
          sha512: (),
          body,
        }))
      },
      | _ => Err(ParseError::Other(
        ParseErrorType::Action,
      )),
    }
  }
}

pub struct Warning {
  warns: u8,
  total: u8,
}

impl Warning {
  pub fn warn(&mut self, msg: String) {
    self.warns += 1;
    if self.warns < self.total {
      let remaining = self.total - self.warns;
      if remaining > 1 {
        warn!("{msg} (this warning will repeat {remaining} more times)");
      } else if remaining == 1 {
        warn!("{msg} (this warning will repeat 1 more time)");
      } else {
        warn!("{msg} (THIS WARNING WILL NOT REPEAT)");
      }
    }
  }

  pub fn new(total: u8) -> Self {
    Self {
      warns: 0,
      total,
    }
  }
}

impl Clone for Warning {
  fn clone(&self) -> Self {
    Self {
      warns: self.warns,
      total: self.total,
    }
  }
}

impl Display for Warning {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Warning: {}/{}",
      self.warns, self.total
    )
  }
}
