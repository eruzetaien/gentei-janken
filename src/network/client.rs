use std::io;
use std::net::{
    IpAddr,
    SocketAddr,
    UdpSocket,
};

use crate::constant::{HOST_PORT, CLIENT_PORT};
use crate::network::{ClientMessage, encode, decode};
pub struct Client {
}

impl Client {
    pub fn start(host_ip: IpAddr, player_name: String) -> io::Result<()> {
        let host_socket = SocketAddr::new(host_ip, HOST_PORT);

        let client_addr = format!("0.0.0.0:{CLIENT_PORT}");
        let socket = UdpSocket::bind(client_addr)?;
        socket.connect(host_socket)?;
        println!("Client connected");

        let join_msg = ClientMessage::Join{player_name};
        let size = socket.send(&encode(&join_msg))?;
        println!("Sent {size} bytes");

        let mut buf = [0u8; 1024];
        loop {
            let (size, addr) = socket.recv_from(&mut buf)?;
            println!("Received {size} bytes from {addr}");
        }
    }
}
