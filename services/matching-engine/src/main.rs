use matchbook_types::*;
use matching_engine::*;
use serde_json;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::io::Split;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;
use tokio::{signal, task};

const DEFAULT_PORT: &str = "50692";
const DEFAULT_MULTICAST: &str = "239.255.42.98";
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let std_socket = bind_multicast(SocketAddrV4::new("0.0.0.0"), multi_addr: &SocketAddrV4)

    let addr = SocketAddrV4::new(IP_ALL.into(), 50692);
    let multi_addr = SocketAddrV4::new(DEFAULT_MULTICAST.parse()?, 50692);
    let std_socket = bind_multicast(&addr, &multi_addr)?;
    let socket = UdpSocket::from_std(std_socket).unwrap();

    // FIXME: how do I make the buffer dynamically grow based on message size?
    let mut buf = vec![0; 1024];

    loop {
        let (n, addr) = socket.recv_from(&mut buf).await?;
        println!("recieved {} bytes from {}", n, addr);

        let order: LimitOrderSubmitRequest = serde_json::from_str(std::str::from_utf8(&buf[..n])?)?;
        println!("order = {:?}", order);
        // socket.send(&buf[..n]).await?;
    }
}

/// Bind socket to multicast address with IP_MULTICAST_LOOP and SO_REUSEADDR Enabled
fn bind_multicast(
    addr: &SocketAddrV4,
    multi_addr: &SocketAddrV4,
) -> Result<std::net::UdpSocket, Box<dyn std::error::Error>> {
    use socket2::{Domain, Protocol, Socket, Type};

    assert!(multi_addr.ip().is_multicast(), "Must be multcast address");

    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;

    socket.set_reuse_address(true)?;
    socket.bind(&socket2::SockAddr::from(*addr))?;
    socket.set_multicast_loop_v4(true)?;
    socket.join_multicast_v4(multi_addr.ip(), addr.ip())?;

    Ok(socket.into_udp_socket())
}
