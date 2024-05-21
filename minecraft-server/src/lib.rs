use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use std::io::{Error, ErrorKind};

#[derive(Debug)]
#[derive(PartialEq)]
pub struct MinecraftServer {
    protocol_version: usize,
    server_version: String,
    motd: String,
    player_count: usize,
    max_players: usize,
}

fn read_null_terminated_string(cursor: &mut Cursor<&[u8]>) -> Result<String, std::io::Error> {    
    let mut string_bytes = vec![];
    let mut next_byte = cursor.read_u16::<BigEndian>()?;
    while next_byte != 0x0000 {
        string_bytes.push(next_byte);
        next_byte = match cursor.read_u16::<BigEndian>(){
            Ok(byte) => byte,
            _ => break,
        };
    }
    match String::from_utf16(&string_bytes) {
        Ok(s) => Ok(s),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
    }
}

pub fn parse_server_list_packet(packet: &[u8]) -> Result<MinecraftServer, Error> {
    let mut cursor = Cursor::new(packet);
    {
        // Check that the packet starts with the correct identifier
        let identifier = cursor.read_u8()?;
        if identifier != 0xff {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid packet identifier, missing 0xff"))
        }

        // Check for string length byte
        let string_length = cursor.read_u16::<BigEndian>()? as usize;
        if packet.len() != string_length*2 + 3 {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid packet identifier: Packet Length incorrect"))
        }

        let identifier2 = cursor.read_u16::<BigEndian>()?;
        let identifier3 = cursor.read_u16::<BigEndian>()?;
        let identifier4 = cursor.read_u16::<BigEndian>()?;
        // Check for §1 pattern 
        if identifier2 != 0x00A7 || identifier3 != 0x0031 || identifier4 != 0x0000 {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid packet identifier: missing §1\u{0}"))
        }
    }

    let protocol_version = read_null_terminated_string(&mut cursor)?.parse::<usize>().expect("Failed to parse usize protocol_version");
    let server_version = read_null_terminated_string(&mut cursor)?;
    let motd = read_null_terminated_string(&mut cursor)?;
    let player_count = read_null_terminated_string(&mut cursor)?.parse::<usize>().expect("Failed to parse usize player_count");
    let max_players= read_null_terminated_string(&mut cursor)?.parse::<usize>().expect("Failed to parse usize max_players");

    Ok(MinecraftServer {
        protocol_version,
        server_version,
        motd,
        player_count,
        max_players,
    })
}

#[test]
fn valid_server_packet() {
    // Data from a 1.19.3 Server with no changes to default settings.
    {
        let packet = vec![0xff, 0, 0x25, 0, 0xa7, 0, 0x31, 0, 0, 0, 49, 0, 50, 0, 55, 0, 0, 0, 49, 0, 46, 0, 49, 0, 57, 0, 46, 0, 51, 0, 0, 0, 65, 0, 32, 0, 77, 0, 105, 0, 110, 0, 101, 0, 99, 0, 114, 0, 97, 0, 102, 0, 116, 0, 32, 0, 83, 0, 101, 0, 114, 0, 118, 0, 101, 0, 114, 0, 0, 0, 48, 0, 0, 0, 50, 0, 48];
        let result = parse_server_list_packet(&packet);

        let expected = MinecraftServer {
            protocol_version: 127,
            server_version: "1.19.3".to_string(),
            motd: "A Minecraft Server".to_string(),
            player_count: 0,
            max_players: 20,
        };
        assert!(!result.is_err(), "This is a valid packet, it should not have errored");
        assert!(expected == result.expect("Should be valid"), "Default server should equal test server");
    }
}

#[test]
fn invalid_packet_identifiers() {
    // Test changing 0xff kick id indicator
    {
        //                 v
        let packet = vec![0xf0, 0, 0x25, 0, 0xa7, 0, 0x31, 0, 0, 0, 49, 0, 50, 0, 55, 0, 0, 0, 49, 0, 46, 0, 49, 0, 57, 0, 46, 0, 51, 0, 0, 0, 65, 0, 32, 0, 77, 0, 105, 0, 110, 0, 101, 0, 99, 0, 114, 0, 97, 0, 102, 0, 116, 0, 32, 0, 83, 0, 101, 0, 114, 0, 118, 0, 101, 0, 114, 0, 0, 0, 48, 0, 0, 0, 50, 0, 48];
        let result = parse_server_list_packet(&packet);
        assert!(result.is_err(), "This packet ID is invalid");
    }
    // Test changing packet length
    {
        //                      v   v
        let packet = vec![0xff, 0, 0x15, 0, 0xa7, 0, 0x31, 0, 0, 0, 49, 0, 50, 0, 55, 0, 0, 0, 49, 0, 46, 0, 49, 0, 57, 0, 46, 0, 51, 0, 0, 0, 65, 0, 32, 0, 77, 0, 105, 0, 110, 0, 101, 0, 99, 0, 114, 0, 97, 0, 102, 0, 116, 0, 32, 0, 83, 0, 101, 0, 114, 0, 118, 0, 101, 0, 114, 0, 0, 0, 48, 0, 0, 0, 50, 0, 48];
        let result = parse_server_list_packet(&packet);
        assert!(result.is_err(), "This packet ID is invalid");
    }
    // Test changing §1\u{0}
    {
        // Changing §
        {
            //                               v   v
            let packet = vec![0xff, 0, 0x25, 0, 0xa0, 0, 0x31, 0, 0, 0, 49, 0, 50, 0, 55, 0, 0, 0, 49, 0, 46, 0, 49, 0, 57, 0, 46, 0, 51, 0, 0, 0, 65, 0, 32, 0, 77, 0, 105, 0, 110, 0, 101, 0, 99, 0, 114, 0, 97, 0, 102, 0, 116, 0, 32, 0, 83, 0, 101, 0, 114, 0, 118, 0, 101, 0, 114, 0, 0, 0, 48, 0, 0, 0, 50, 0, 48];
            let result = parse_server_list_packet(&packet);
            assert!(result.is_err(), "This packet ID is invalid");
        }
        // Changing 1
        {
            //                                        v   v
            let packet = vec![0xff, 0, 0x25, 0, 0xa7, 0, 0x21, 0, 0, 0, 49, 0, 50, 0, 55, 0, 0, 0, 49, 0, 46, 0, 49, 0, 57, 0, 46, 0, 51, 0, 0, 0, 65, 0, 32, 0, 77, 0, 105, 0, 110, 0, 101, 0, 99, 0, 114, 0, 97, 0, 102, 0, 116, 0, 32, 0, 83, 0, 101, 0, 114, 0, 118, 0, 101, 0, 114, 0, 0, 0, 48, 0, 0, 0, 50, 0, 48];
            let result = parse_server_list_packet(&packet);
            assert!(result.is_err(), "This packet ID is invalid");
        }
        // Changing \u{0}
        {
            //                                                  v     v
            let packet = vec![0xff, 0, 0x25, 0, 0xa0, 0, 0x31, 0x01, 0x00, 0, 49, 0, 50, 0, 55, 0, 0, 0, 49, 0, 46, 0, 49, 0, 57, 0, 46, 0, 51, 0, 0, 0, 65, 0, 32, 0, 77, 0, 105, 0, 110, 0, 101, 0, 99, 0, 114, 0, 97, 0, 102, 0, 116, 0, 32, 0, 83, 0, 101, 0, 114, 0, 118, 0, 101, 0, 114, 0, 0, 0, 48, 0, 0, 0, 50, 0, 48];
            let result = parse_server_list_packet(&packet);
            assert!(result.is_err(), "This packet ID is invalid");
        }
    }
}
