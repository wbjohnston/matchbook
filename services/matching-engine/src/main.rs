use matchbook_types::{Command, Price, Quantity};
use matching_engine::*;
use serde_json;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::UdpSocket;

const DEFAULT_MULTICAST_ADDRESS: &'static str = "239.255.42.98";
const DEFAULT_MULTICAST_PORT: &'static str = "50692";
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = get_config_from_env()?;

    let addr = SocketAddrV4::new(IP_ALL.into(), config.multicast_port);
    let multi_addr = SocketAddrV4::new(config.multicast_address, config.multicast_port);
    let std_socket = bind_multicast(&addr, &multi_addr)?;
    let socket = UdpSocket::from_std(std_socket)?;

    // TODO(will): how do I make the buffer dynamically grow based on message size?
    let mut buf = vec![0; 1024 * 1024];

    let mut engine = MatchingEngine::default();
    engine.create_symbol("foo".to_string());

    loop {
        let (n, addr) = socket.recv_from(&mut buf).await?;
        println!("recieved {} bytes from {}", n, addr);

        let message_as_str = match std::str::from_utf8(&buf[..n]) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("failed to parse received message into utf8. ignoring message");
                eprintln!("{}", e);
                continue;
            }
        };

        let command: Command = match serde_json::from_str(message_as_str) {
            Ok(command) => command,
            Err(e) => {
                eprintln!("failed to parse message into command");
                eprintln!("{}", e);
                continue;
            }
        };

        println!("order = {:?}", &command);
        match command {
            Command::LimitOrderSubmitRequest {
                quantity,
                price,
                symbol,
                side,
            } => {
                let fills = engine.submit_limit_order(side, symbol.as_str(), price, quantity)?;
                dbg!(&fills);
            }
        }

        dbg!(&engine);
    }
}

#[derive(Debug, Clone, Copy)]
struct Config {
    pub multicast_address: Ipv4Addr,
    pub multicast_port: u16,
}

fn get_config_from_env() -> Result<Config, Box<dyn std::error::Error>> {
    Ok(Config {
        multicast_address: std::env::var("MULTICAST_ADDRESS")
            .unwrap_or(DEFAULT_MULTICAST_ADDRESS.to_string())
            .parse()?,
        multicast_port: std::env::var("MULTICAST_PORT")
            .unwrap_or(DEFAULT_MULTICAST_PORT.to_string())
            .parse()?,
    })
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
