extern crate minecraft_server;

use std::io::{prelude::*};
use std::net::{TcpStream, SocketAddr};
use std::io::{self, BufRead};

fn main() {


    let server_addr = SocketAddr::from(([0, 0, 0, 0], 25565));

    let mut socket = TcpStream::connect(server_addr).expect("Could not connect to address.");

    // This is the slightly newer packet, it does not work prior to version 1.4
    socket.write(&[0xFE, 0x01]).expect("Failed to send message.");

    let mut test = io::BufReader::new(socket);

    let buf: &mut Vec<u8> = &mut test.fill_buf().expect("Failed to read data.").to_vec();

    let server = minecraft_server::parse_server_list_packet(buf);

    println!("{:?}", server);
}
