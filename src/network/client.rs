use std::io;
use std::net::{
    IpAddr,
    SocketAddr,
    UdpSocket,
};

use crate::constant::HOST_PORT;
use crate::constant::CLIENT_PORT;

pub struct Client {
}

impl Client {
    pub fn start(host_ip: IpAddr) -> io::Result<()> {
        let host_socket = SocketAddr::new(host_ip, HOST_PORT);

        let client_addr = format!("0.0.0.0:{CLIENT_PORT}");
        let socket = UdpSocket::bind(client_addr)?;
        socket.connect(host_socket)?;
        println!("Client connected");

        let message = b"Hello from client!";
        let size = socket.send(message)?;
        println!("Sent {size} bytes");

        let mut buf = [0u8; 1024];
        loop {
            let (size, addr) = socket.recv_from(&mut buf)?;
            println!("Received {size} bytes from {addr}");
        }
    }
}
