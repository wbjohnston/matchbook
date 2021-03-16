#![deny(clippy::all)]
use std::net::SocketAddrV4;
pub mod codec;
pub use codec::*;

/// Bind socket to multicast address with IP_MULTICAST_LOOP and SO_REUSEADDR Enabled
pub fn bind_multicast(
    addr: &SocketAddrV4,
    multi_addr: &SocketAddrV4,
) -> Result<std::net::UdpSocket, Box<dyn std::error::Error>> {
    use socket2::{Domain, Protocol, Socket, Type};

    assert!(multi_addr.ip().is_multicast(), "Must be multcast address");

    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;

    socket.set_reuse_address(true)?;
    socket.bind(&socket2::SockAddr::from(*addr))?;
    socket.set_nonblocking(true)?;
    socket.set_multicast_loop_v4(false)?;
    socket.join_multicast_v4(multi_addr.ip(), addr.ip())?;

    Ok(socket.into_udp_socket())
}
