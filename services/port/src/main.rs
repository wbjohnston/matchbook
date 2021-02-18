use env_logger;
use log;
use matchbook_types::*;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, UdpSocket};

const DEFAULT_MULTICAST_ADDRESS: &'static str = "239.255.42.98";
const DEFAULT_MULTICAST_PORT: &'static str = "50692";
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let config = get_config_from_env()?;
    let tcp_listener_addr: SocketAddrV4 = format!("127.0.0.1:{}", 8080).parse()?;
    let tcp_listener = TcpListener::bind(tcp_listener_addr).await?;
    log::info!("started listening for TCP clients on {}", tcp_listener_addr);

    let addr = SocketAddrV4::new(IP_ALL.into(), config.multicast_port);
    let multi_addr = SocketAddrV4::new(config.multicast_address, config.multicast_port);
    let std_socket = bind_multicast(&addr, &multi_addr)?;
    let udp_socket = UdpSocket::from_std(std_socket)?;
    log::info!("bound to multicast on {}", udp_socket.local_addr()?);

    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    tokio::spawn(async move {
        while let Some(command) = rx.recv().await {
            match udp_socket
                .send_to(
                    serde_json::to_string(&command).unwrap().as_bytes(),
                    multi_addr,
                )
                .await
            {
                Ok(_) => {
                    log::info!("successfully forwarded message to udp backbone");
                }
                Err(e) => {
                    log::error!("failed to send message to udp backbone: {:?}", e);
                }
            }
        }
    });

    loop {
        let (mut socket, addr) = tcp_listener.accept().await?;
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    _ => unimplemented!(),
                };

                log::info!("read {} bytes from {}", n, addr);

                let st = std::str::from_utf8(&buf[..n]).unwrap();
                let command: Command = serde_json::from_str(st).unwrap();

                {
                    match &command {
                        Command::LimitOrderSubmitRequest {
                            symbol,
                            price,
                            quantity,
                            side,
                        } => {
                            log::info!(
                                "received limit order submit request to {:?} {} {} at {} from client at {}",
                                side,
                                symbol,
                                quantity,
                                price,
                                addr
                            );
                        }
                    }
                }

                match tx.send(command).await {
                    Ok(_) => {
                        log::info!("successfully forwarded message UDP message handling thread")
                    }
                    Err(e) => log::error!(
                        "failed to forward message to UDP message handling thread: {}",
                        e
                    ),
                }
            }
        });
    }
}

/// Bind socket to multicast address with IP_MULTICAST_LOOP and SO_REUSEADDR Enabled
fn bind_multicast(
    addr: &SocketAddrV4,
    multi_addr: &SocketAddrV4,
) -> Result<std::net::UdpSocket, Box<dyn std::error::Error>> {
    assert!(multi_addr.ip().is_multicast(), "Must be multicast address");

    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;

    socket.set_reuse_address(true)?;
    socket.bind(&socket2::SockAddr::from(*addr))?;
    socket.set_multicast_loop_v4(true)?;
    socket.join_multicast_v4(multi_addr.ip(), addr.ip())?;

    Ok(socket.into_udp_socket())
}

#[derive(Debug, Clone, Copy)]
struct Config {
    pub service_identifier: ServiceIdentifier,
    pub multicast_address: Ipv4Addr,
    pub multicast_port: u16,
}

fn get_config_from_env() -> Result<Config, Box<dyn std::error::Error>> {
    Ok(Config {
        service_identifier: std::env::var("SERVICE_IDENTIFIER")
            .map(|x| ServiceIdentifier::from_str(x.as_str()).unwrap())
            .expect("missing required environment variable 'SERVICE_IDENTIFIER'"),

        multicast_address: std::env::var("MULTICAST_ADDRESS")
            .unwrap_or(DEFAULT_MULTICAST_ADDRESS.to_string())
            .parse()?,

        multicast_port: std::env::var("MULTICAST_PORT")
            .unwrap_or(DEFAULT_MULTICAST_PORT.to_string())
            .parse()?,
    })
}
