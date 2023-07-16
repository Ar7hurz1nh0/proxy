#[allow(unused_imports)]
use crate::utils::{
  hash_sha1, hash_sha512, split, Client, Packet, PacketAction, PacketType,
  Server,
};
#[allow(unused_imports)]
use std::str::FromStr;
#[allow(unused_imports)]
use uuid::Uuid;

#[test]
fn split_big() {
  println!(
    "Full packet: {} {} {} {} {} {} {} {} {} {} {} {}",
    0x9, 0x8, 0x7, 0x4, 0x2, 0x0, 0x0, 0x0, 0x2, 0x4, 0xA, 0xF
  );
  println!("Separator: {} {} {}", 0x0, 0x0, 0x0);
  let packet: Vec<u8> =
    vec![0x9, 0x8, 0x7, 0x4, 0x2, 0x0, 0x0, 0x0, 0x2, 0x4, 0xA, 0xF];
  let separator: Vec<u8> = vec![0x0, 0x0, 0x0];
  let result = split(&packet, &separator);
  assert_eq!(result.is_some(), true);
  if let Some(result) = result {
    println!(
      "Expected: {} {} {} {} {}",
      0x9, 0x8, 0x7, 0x4, 0x2
    );
    println!(
      "Got: {}",
      result
        .0
        .iter()
        .map(|byte| byte.to_string())
        .collect::<Vec<String>>()
        .join(" ")
    );
    assert_eq!(result.0, vec![0x9, 0x8, 0x7, 0x4, 0x2]);
    println!(
      "Expected: {} {} {} {}",
      0x2, 0x4, 0xA, 0xF
    );
    println!(
      "Got: {}",
      result
        .1
        .iter()
        .map(|byte| byte.to_string())
        .collect::<Vec<String>>()
        .join(" ")
    );
    assert_eq!(result.1, vec![0x2, 0x4, 0xA, 0xF]);
  } else {
    assert_eq!(true, false, "Got: None");
  }
}

#[test]
fn split_byte() {
  println!(
    "Full packet: {} {} {} {} {} {} {} {} {} {} {} {}",
    0x9, 0x8, 0x7, 0x4, 0x2, 0x0, 0x0, 0x0, 0x2, 0x4, 0xA, 0xF
  );
  println!("Separator: {}", 0x0);
  let packet: Vec<u8> =
    vec![0x9, 0x8, 0x7, 0x4, 0x2, 0x0, 0x0, 0x0, 0x2, 0x4, 0xA, 0xF];
  let separator: Vec<u8> = vec![0x0];
  let result = split(&packet, &separator);
  assert_eq!(result.is_some(), true);
  if let Some(result) = result {
    println!(
      "Expected: {} {} {} {} {}",
      0x9, 0x8, 0x7, 0x4, 0x2
    );
    println!(
      "Got: {}",
      result
        .0
        .iter()
        .map(|byte| byte.to_string())
        .collect::<Vec<String>>()
        .join(" ")
    );
    assert_eq!(result.0, vec![0x9, 0x8, 0x7, 0x4, 0x2]);
    println!(
      "Expected 1: {} {} {} {} {} {}",
      0x0, 0x0, 0x2, 0x4, 0xA, 0xF
    );
    println!(
      "Got: {}",
      result
        .1
        .iter()
        .map(|byte| byte.to_string())
        .collect::<Vec<String>>()
        .join(" ")
    );
    assert_eq!(
      result.1,
      vec![0x0, 0x0, 0x2, 0x4, 0xA, 0xF]
    );
  } else {
    assert_eq!(true, false, "Got: None");
  }
}

#[test]
fn split_more() {
  println!(
    "Full packet: {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
    0x9, 0x8, 0x7, 0x4, 0x2, 0x0, 0x0, 0x0, 0x2, 0x4, 0xA, 0xF, 0x0, 0x0, 0x0, 0x7, 0x4, 0x2,0x0, 0x4, 0xA, 0xF, 0x0, 0x0, 0x0
  );
  println!("Separator: {} {} {}", 0x0, 0x0, 0x0);
  let packet: Vec<u8> = vec![
    0x9, 0x8, 0x7, 0x4, 0x2, 0x0, 0x0, 0x0, 0x2, 0x4, 0xA, 0xF, 0x0, 0x0, 0x0,
    0x7, 0x4, 0x2, 0x0, 0x4, 0xA, 0xF, 0x0, 0x0, 0x0,
  ];
  let separator: Vec<u8> = vec![0x0, 0x0, 0x0];
  let result = split(&packet, &separator);
  assert_eq!(result.is_some(), true);
  if let Some(result) = result {
    println!(
      "Expected: {} {} {} {} {}",
      0x9, 0x8, 0x7, 0x4, 0x2
    );
    println!(
      "Got: {}",
      result
        .0
        .iter()
        .map(|byte| byte.to_string())
        .collect::<Vec<String>>()
        .join(" ")
    );
    assert_eq!(result.0, vec![0x9, 0x8, 0x7, 0x4, 0x2]);
    println!(
      "Expected: {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
      0x2,
      0x4,
      0xA,
      0xF,
      0x0,
      0x0,
      0x0,
      0x7,
      0x4,
      0x2,
      0x0,
      0x4,
      0xA,
      0xF,
      0x0,
      0x0,
      0x0
    );

    println!(
      "Got: {}",
      result
        .1
        .iter()
        .map(|byte| byte.to_string())
        .collect::<Vec<String>>()
        .join(" ")
    );
    assert_eq!(
      result.1,
      vec![
        0x2, 0x4, 0xA, 0xF, 0x0, 0x0, 0x0, 0x7, 0x4, 0x2, 0x0, 0x4, 0xA, 0xF,
        0x0, 0x0, 0x0
      ]
    );
  } else {
    assert_eq!(true, false, "Got: None");
  }
}

#[test]
fn split_none() {
  println!(
    "Full packet: {} {} {} {} {} {} {} {} {} {} {} {}",
    0x9, 0x8, 0x7, 0x4, 0x2, 0x0, 0x0, 0x0, 0x2, 0x4, 0xA, 0xF
  );
  println!("Separator: {} {}", 0x0, 0x1);
  println!("Expected: None");
  let packet: Vec<u8> =
    vec![0x9, 0x8, 0x7, 0x4, 0x2, 0x0, 0x0, 0x0, 0x2, 0x4, 0xA, 0xF];
  let separator: Vec<u8> = vec![0x0, 0x1];
  let result = split(&packet, &separator);
  if result.is_none() {
    println!("Got: None");
  } else {
    println!("Got: Some")
  }
  assert_eq!(result.is_none(), true);
}

#[test]
fn split_to_none() {
  println!(
    "Full packet: {} {} {} {} {} {} {}",
    0x9, 0x8, 0x7, 0x4, 0x2, 0x0, 0x0
  );
  println!("Separator: {} {}", 0x0, 0x0);
  let packet: Vec<u8> = vec![0x9, 0x8, 0x7, 0x4, 0x2, 0x0, 0x0];
  let separator: Vec<u8> = vec![0x0, 0x0];
  let result = split(&packet, &separator);
  assert_eq!(result.is_some(), true);
  if let Some(result) = result {
    println!(
      "Expected: {} {} {} {} {}",
      0x9, 0x8, 0x7, 0x4, 0x2
    );
    println!(
      "Got: {}",
      result
        .0
        .iter()
        .map(|byte| byte.to_string())
        .collect::<Vec<String>>()
        .join(" ")
    );
    assert_eq!(result.0, vec![0x9, 0x8, 0x7, 0x4, 0x2]);
    println!("Expected: None");
    println!(
      "Got: {}",
      result
        .1
        .iter()
        .map(|byte| byte.to_string())
        .collect::<Vec<String>>()
        .join(" ")
    );
    let res: Vec<u8> = vec![];
    assert_eq!(result.1, res);
  } else {
    assert_eq!(true, false, "Got: None");
  }
}

#[test]
fn split_some_to_none() {
  println!(
    "Full packet: {} {} {} {} {} {} {}",
    0x0, 0x0, 0x9, 0x8, 0x7, 0x4, 0x2
  );
  println!("Separator: {} {}", 0x0, 0x0);
  let packet: Vec<u8> = vec![0x0, 0x0, 0x9, 0x8, 0x7, 0x4, 0x2];
  let separator: Vec<u8> = vec![0x0, 0x0];
  let result = split(&packet, &separator);
  assert_eq!(result.is_some(), true);
  if let Some(result) = result {
    println!("Expected: None");
    println!(
      "Got: {}",
      result
        .0
        .iter()
        .map(|byte| byte.to_string())
        .collect::<Vec<String>>()
        .join(" ")
    );
    let res: Vec<u8> = vec![];
    assert_eq!(result.0, res);
    println!(
      "Expected: {} {} {} {} {}",
      0x9, 0x8, 0x7, 0x4, 0x2
    );
    println!(
      "Got: {}",
      result
        .1
        .iter()
        .map(|byte| byte.to_string())
        .collect::<Vec<String>>()
        .join(" ")
    );
    assert_eq!(result.1, vec![0x9, 0x8, 0x7, 0x4, 0x2]);
  } else {
    assert_eq!(true, false, "Got: None");
  }
}

#[test]
fn auth_packet() {
  let packet_test = Client::build_auth_packet(
    &String::from("123").into_bytes(),
    &vec![3000, 4000, 5000],
    &String::from("\u{0000}").into_bytes(),
  );

  let packet = vec![
    0x41, 0x55, 0x54, 0x48, 0x20, 0x33, 0x30, 0x30, 0x30, 0x2C, 0x34, 0x30,
    0x30, 0x30, 0x2C, 0x35, 0x30, 0x30, 0x30, 0x0, 0x31, 0x32, 0x33,
  ];

  assert_eq!(packet_test.unwrap(), packet);
}

#[test]
fn data_packet_client() {
  let id = "8c95a08a-97d1-4330-b5bf-87866baae5de";
  let id = Uuid::from_str(id).unwrap();
  let data = vec![0x0, 0x01, 0x26, 0x42, 0xAF, 0xFF];
  let packet_test = Client::build_data_packet(
    &id,
    &"\u{0000}".as_bytes().to_vec(),
    &data.clone(),
  );

  let sha1_hash = hash_sha1(&data).as_bytes().to_vec();
  let sha512_hash = hash_sha512(&data).as_bytes().to_vec();
  let mut packet = PacketAction::DATA.value().as_bytes().to_vec();
  packet.extend(vec![0x20]);
  packet.extend(format!("{id}").as_bytes().to_vec());
  packet.extend(vec![0x20]);
  packet.extend(sha1_hash);
  packet.extend(vec![0x20]);
  packet.extend(sha512_hash);
  packet.extend(vec![0x00]);
  packet.extend(vec![0x00, 0x01, 0x26, 0x42, 0xAF, 0xFF]);

  assert_eq!(packet_test.unwrap(), packet);
}

#[test]
fn data_packet_server() {
  let id = "8c95a08a-97d1-4330-b5bf-87866baae5de";
  let id = Uuid::from_str(id).unwrap();
  let data = vec![0x0, 0x01, 0x26, 0x42, 0xAF, 0xFF];
  let packet_test = Server::build_data_packet(
    &id,
    &3000,
    &"\u{0000}".as_bytes().to_vec(),
    &data.clone(),
  );

  let sha1_hash = hash_sha1(&data).as_bytes().to_vec();
  let sha512_hash = hash_sha512(&data).as_bytes().to_vec();
  let mut packet: Vec<u8> = PacketAction::DATA.value().as_bytes().to_vec();
  packet.extend(vec![0x20]);
  packet.extend(format!("{id}").as_bytes().to_vec()); // ID
  packet.extend(vec![0x20]);
  packet.extend(vec![0x33, 0x30, 0x30, 0x30]); // Port
  packet.extend(vec![0x20]);
  packet.extend(sha1_hash); // SHA1
  packet.extend(vec![0x20]);
  packet.extend(sha512_hash); // SHA512
  packet.extend(vec![0x00]);
  packet.extend(vec![0x00, 0x01, 0x26, 0x42, 0xAF, 0xFF]);

  assert_eq!(packet_test.unwrap(), packet);
}

#[test]
fn sha1() {
  let hash_test = hash_sha1(&vec![0x31, 0x32, 0x33]);

  let hash = String::from("40bd001563085fc35165329ea1ff5c5ecbdbbeef");

  assert_eq!(
    hash_test.to_lowercase(),
    hash.to_lowercase()
  );
}

#[test]
fn sha1_empty() {
  let hash_test = hash_sha1(&vec![]);

  let hash = String::from("da39a3ee5e6b4b0d3255bfef95601890afd80709");

  assert_eq!(
    hash_test.to_lowercase(),
    hash.to_lowercase()
  );
}

#[test]
fn sha512() {
  let hash_test = hash_sha512(&vec![0x31, 0x32, 0x33]);

  let hash = String::from("3c9909afec25354d551dae21590bb26e38d53f2173b8d3dc3eee4c047e7ab1c1eb8b85103e3be7ba613b31bb5c9c36214dc9f14a42fd7a2fdb84856bca5c44c2");

  assert_eq!(
    hash_test.to_lowercase(),
    hash.to_lowercase()
  );
}

#[test]
fn sha512_empty() {
  let hash_test = hash_sha512(&vec![]);

  let hash = String::from("cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e");

  assert_eq!(
    hash_test.to_lowercase(),
    hash.to_lowercase()
  );
}

#[test]
fn parse_data_client() {
  let id = "8c95a08a-97d1-4330-b5bf-87866baae5de";
  let id = Uuid::from_str(id).unwrap();
  let port: u16 = 3000;
  let data = vec![0x0, 0x01, 0x26, 0x42, 0xAF, 0xFF];
  let sha1_hash = hash_sha1(&data);
  let sha512_hash = hash_sha512(&data);
  let separator: Vec<u8> = vec![0x00];
  let mut packet = PacketAction::DATA.value().as_bytes().to_vec();
  packet.extend(vec![0x20]);
  packet.extend(format!("{id}").as_bytes().to_vec());
  packet.extend(vec![0x20]);
  packet.extend(format!("{port}").as_bytes().to_vec());
  packet.extend(vec![0x20]);
  packet.extend(sha1_hash.as_bytes().to_vec());
  packet.extend(vec![0x20]);
  packet.extend(sha512_hash.as_bytes().to_vec());
  packet.extend(separator.clone());
  packet.extend(data.clone());

  match Client::parse_packet(&packet.clone(), &separator) {
    | Ok(packet_test) => match packet_test {
      | PacketType::Data(packet_test) => {
        assert_eq!(packet_test.id, id);
        assert_eq!(packet_test.port, port);
        assert_eq!(packet_test.ports, ());
        assert_eq!(packet_test.sha1, sha1_hash);
        assert_eq!(packet_test.sha512, sha512_hash);
        assert_eq!(packet_test.body, data);
      },
      | _ => panic!("Packet is not a data packet"),
    },
    | Err(err) => panic!("{err}"),
  }
}

#[test]
fn parse_auth_client() {
  let id = "8c95a08a-97d1-4330-b5bf-87866baae5de";
  let id = Uuid::from_str(id).unwrap();
  let ports: Vec<u16> = vec![6753, 11, 6, 9, 4, 2, 8];
  let data = vec![0x0, 0x01, 0x26, 0x42, 0xAF, 0xFF];
  let separator: Vec<u8> = vec![0x00];
  let mut packet = PacketAction::AUTH.value().as_bytes().to_vec();
  packet.extend(vec![0x20]);
  packet.extend(format!("{id}").as_bytes().to_vec());
  packet.extend(vec![0x20]);
  packet.extend(
    ports
      .iter()
      .map(|x| x.to_string())
      .collect::<Vec<String>>()
      .join(",")
      .as_bytes()
      .to_vec(),
  );
  packet.extend(separator.clone());
  packet.extend(data.clone());

  match Client::parse_packet(&packet.clone(), &separator) {
    | Ok(_) => panic!("Packet should not be parsed"),
    | _ => (),
  }
}

#[test]
fn parse_close_client() {
  let id = "8c95a08a-97d1-4330-b5bf-87866baae5de";
  let id = Uuid::from_str(id).unwrap();
  let data = vec![];
  let separator: Vec<u8> = vec![0x00];
  let mut packet = PacketAction::CLOSE.value().as_bytes().to_vec();
  packet.extend(vec![0x20]);
  packet.extend(format!("{id}").as_bytes().to_vec());
  packet.extend(separator.clone());

  match Client::parse_packet(&packet.clone(), &separator) {
    | Ok(packet_test) => match packet_test {
      | PacketType::Close(packet_test) => {
        assert_eq!(packet_test.id, id);
        assert_eq!(packet_test.port, 0);
        assert_eq!(packet_test.ports, ());
        assert_eq!(packet_test.sha1, ());
        assert_eq!(packet_test.sha512, ());
        assert_eq!(packet_test.body, data);
      },
      | _ => panic!("Packet is not a data packet"),
    },
    | Err(err) => panic!("{err}"),
  }
}

#[test]
fn parse_data_server() {
  let id = "8c95a08a-97d1-4330-b5bf-87866baae5de";
  let id = Uuid::from_str(id).unwrap();
  let data = vec![0x0, 0x01, 0x26, 0x42, 0xAF, 0xFF];
  let sha1_hash = hash_sha1(&data);
  let sha512_hash = hash_sha512(&data);
  let separator: Vec<u8> = vec![0x00];
  let mut packet = PacketAction::DATA.value().as_bytes().to_vec();
  packet.extend(vec![0x20]);
  packet.extend(format!("{id}").as_bytes().to_vec());
  packet.extend(vec![0x20]);
  packet.extend(sha1_hash.as_bytes().to_vec());
  packet.extend(vec![0x20]);
  packet.extend(sha512_hash.as_bytes().to_vec());
  packet.extend(separator.clone());
  packet.extend(data.clone());

  match Server::parse_packet(&packet, &separator) {
    | Ok(packet_test) => match packet_test {
      | PacketType::Data(packet_test) => {
        assert_eq!(packet_test.id, id);
        assert_eq!(packet_test.port, ());
        assert_eq!(packet_test.ports, ());
        assert_eq!(packet_test.sha1, sha1_hash);
        assert_eq!(packet_test.sha512, sha512_hash);
        assert_eq!(packet_test.body, data);
      },
      | _ => panic!("Packet is not a data packet"),
    },
    | Err(err) => panic!("{err}"),
  }
}

#[test]
fn parse_auth_server() {
  let ports: Vec<u16> = vec![6753, 11, 6, 9, 4, 2, 8];
  let data = vec![0x0, 0x01, 0x26, 0x42, 0xAF, 0xFF];
  let separator: Vec<u8> = vec![0x00];
  let mut packet = PacketAction::AUTH.value().as_bytes().to_vec();
  packet.extend(vec![0x20]);
  packet.extend(
    ports
      .iter()
      .map(|x| x.to_string())
      .collect::<Vec<String>>()
      .join(",")
      .as_bytes()
      .to_vec(),
  );
  packet.extend(separator.clone());
  packet.extend(data.clone());

  println!(
    "{}",
    ports.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",")
  );

  match Server::parse_packet(&packet, &separator) {
    | Ok(packet_test) => match packet_test {
      | PacketType::Auth(packet_test) => {
        assert_eq!(packet_test.id, ());
        assert_eq!(packet_test.port, ());
        assert_eq!(packet_test.ports, ports);
        assert_eq!(packet_test.sha1, ());
        assert_eq!(packet_test.sha512, ());
        assert_eq!(packet_test.body, data);
      },
      | _ => panic!("Packet is not a data packet"),
    },
    | Err(err) => panic!("{err}"),
  }
}

#[test]
fn parse_close_server() {
  let id = "8c95a08a-97d1-4330-b5bf-87866baae5de";
  let id = Uuid::from_str(id).unwrap();
  let separator: Vec<u8> = vec![0x00];
  let data: Vec<u8> = vec![];
  let mut packet = PacketAction::CLOSE.value().as_bytes().to_vec();
  packet.extend(vec![0x20]);
  packet.extend(format!("{id}").as_bytes().to_vec());
  packet.extend(separator.clone());

  match Server::parse_packet(&packet, &separator) {
    | Ok(packet_test) => match packet_test {
      | PacketType::Close(packet_test) => {
        assert_eq!(packet_test.id, id);
        assert_eq!(packet_test.port, ());
        assert_eq!(packet_test.ports, ());
        assert_eq!(packet_test.sha1, ());
        assert_eq!(packet_test.sha512, ());
        assert_eq!(packet_test.body, data);
      },
      | _ => panic!("Packet is not a data packet"),
    },
    | Err(err) => panic!("{err}"),
  }
}

#[test]
fn build_to_parse_client_data() {
  let id = Uuid::new_v4();
  let separator = "\u{0000}".as_bytes().to_vec();
  let data = vec![0x0, 0x01, 0x26, 0x42, 0xAF, 0xFF];
  let packet = Client::build_data_packet(&id, &separator, &data);

  let packet = Server::parse_packet(&packet.unwrap(), &separator).unwrap();

  match packet {
    | PacketType::Data(packet) => {
      assert_eq!(packet.id, id);
      assert_eq!(packet.port, ());
      assert_eq!(packet.ports, ());
      assert_eq!(packet.sha1, hash_sha1(&data));
      assert_eq!(packet.sha512, hash_sha512(&data));
      assert_eq!(packet.body, data);
    },
    | _ => panic!("Packet is not a data packet"),
  }
}

#[test]
fn build_to_parse_client_auth() {
  let separator = "\u{0000}".as_bytes().to_vec();
  let auth = String::from("(*HN)PIu)*&(hBI").into_bytes();
  let ports: Vec<u16> = vec![6753, 11, 6, 9, 4, 2, 8];
  let packet = Client::build_auth_packet(&auth, &ports, &separator);

  let packet = Server::parse_packet(&packet.unwrap(), &separator).unwrap();

  match packet {
    | PacketType::Auth(packet) => {
      assert_eq!(packet.id, ());
      assert_eq!(packet.port, ());
      assert_eq!(packet.ports, ports);
      assert_eq!(packet.sha1, ());
      assert_eq!(packet.sha512, ());
      assert_eq!(packet.body, auth);
    },
    | _ => panic!("Packet is not a data packet"),
  }
}

#[test]
fn build_to_parse_client_close() {
  let id = Uuid::new_v4();
  println!("{id}");
  let separator = "\u{0000}".as_bytes().to_vec();
  let data = vec![];
  let packet = Client::close_connection_packet(&id, &separator);

  let packet = Server::parse_packet(&packet.unwrap(), &separator).unwrap();

  match packet {
    | PacketType::Close(packet) => {
      assert_eq!(packet.id, id);
      assert_eq!(packet.port, ());
      assert_eq!(packet.ports, ());
      assert_eq!(packet.sha1, ());
      assert_eq!(packet.sha512, ());
      assert_eq!(packet.body, data);
    },
    | _ => panic!("Packet is not a data packet"),
  }
}

#[test]
fn build_to_parse_server_data() {
  let id = Uuid::new_v4();
  let separator = "\u{0000}".as_bytes().to_vec();
  let port: u16 = 6753;
  let data = vec![0x0, 0x01, 0x26, 0x42, 0xAF, 0xFF];
  let packet = Server::build_data_packet(&id, &port, &separator, &data);

  let packet = Client::parse_packet(&packet.unwrap(), &separator).unwrap();

  match packet {
    | PacketType::Data(packet) => {
      assert_eq!(packet.id, id);
      assert_eq!(packet.port, port);
      assert_eq!(packet.ports, ());
      assert_eq!(packet.sha1, hash_sha1(&data));
      assert_eq!(packet.sha512, hash_sha512(&data));
      assert_eq!(packet.body, data);
    },
    | _ => panic!("Packet is not a data packet"),
  }
}

#[test]
fn build_to_parse_server_close() {
  let id = Uuid::new_v4();
  let separator = "\u{0000}".as_bytes().to_vec();
  let data: Vec<u8> = vec![];
  let packet = Server::close_connection_packet(&id, &separator);

  let packet = Client::parse_packet(&packet.unwrap(), &separator).unwrap();

  match packet {
    | PacketType::Close(packet) => {
      assert_eq!(packet.id, id);
      assert_eq!(packet.port, 0);
      assert_eq!(packet.ports, ());
      assert_eq!(packet.sha1, ());
      assert_eq!(packet.sha512, ());
      assert_eq!(packet.body, data);
    },
    | _ => panic!("Packet is not a data packet"),
  }
}
