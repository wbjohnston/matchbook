#![deny(clippy::all)]
use futures::StreamExt;
use futures::{Sink, SinkExt, Stream};
use matchbook_types::Message;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;

const IP_ALL: [u8; 4] = [0, 0, 0, 0];

pub mod codec;
pub use codec::*;

/// Bind socket to multicast address with IP_MULTICAST_LOOP and SO_REUSEADDR Enabled
pub fn bind_multicast(
    multi_addr: &SocketAddr,
) -> Result<std::net::UdpSocket, Box<dyn std::error::Error>> {
    use socket2::{Domain, Protocol, Socket, Type};

    assert!(multi_addr.ip().is_multicast(), "Must be multcast address");

    let addr = SocketAddr::new(IP_ALL.into(), multi_addr.port());
    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;

    socket.set_reuse_address(true)?;
    socket.bind(&socket2::SockAddr::from(addr))?;
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

pub fn make_matchbook_streams(
    multi_addr: SocketAddr,
) -> Result<
    (
        impl Sink<Message, Error = std::io::Error> + std::fmt::Debug,
        impl Stream<Item = Result<Message, std::io::Error>> + std::fmt::Debug,
    ),
    Box<dyn std::error::Error>,
> {
    let socket = bind_multicast(&multi_addr)?;
    let socket = UdpSocket::from_std(socket)?;
    let (sink, stream) = UdpFramed::new(socket, MatchbookMessageCodec::new()).split();
    let sink = sink.with(move |x| futures::future::ok((x, multi_addr)));
    let stream = stream.map(|x| x.map(|(y, _)| y));

    Ok((sink, stream))
}
