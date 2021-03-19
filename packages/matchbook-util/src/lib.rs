#![deny(clippy::all)]
use std::net::SocketAddr;
pub mod codec;
pub use codec::*;

/// Bind socket to multicast address with IP_MULTICAST_LOOP and SO_REUSEADDR Enabled
pub fn bind_multicast(
    addr: &SocketAddr,
    multi_addr: &SocketAddr,
) -> Result<std::net::UdpSocket, Box<dyn std::error::Error>> {
    use socket2::{Domain, Protocol, Socket, Type};

    assert!(multi_addr.ip().is_multicast(), "Must be multcast address");

    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;

    socket.set_reuse_address(true)?;
    socket.bind(&socket2::SockAddr::from(*addr))?;
    socket.set_nonblocking(true)?;
    socket.set_multicast_loop_v4(false)?;

    match (multi_addr, addr) {
        (SocketAddr::V4(multi_addr), SocketAddr::V4(addr)) => {
            socket.join_multicast_v4(multi_addr.ip(), addr.ip())?;
        }
        _ => todo!(),
    }

    Ok(socket.into_udp_socket())
}
