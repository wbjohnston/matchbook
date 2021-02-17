use matchbook_types::*;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, UdpSocket};

const DEFAULT_MULTICAST_ADDRESS: &'static str = "239.255.42.98";
const DEFAULT_MULTICAST_PORT: &'static str = "50692";
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = get_config_from_env()?;
    dbg!(&config);
    let tcp_listener_addr: SocketAddrV4 = format!("127.0.0.1:{}", 8080).parse()?;
    let tcp_listener = TcpListener::bind(tcp_listener_addr).await?;
    println!("started listening at address = {}", tcp_listener_addr);

    println!(
        "starting udp multicast {} emmitter at {}",
        config.multicast_address, config.multicast_port
    );
    let addr = SocketAddrV4::new(IP_ALL.into(), config.multicast_port);
    let multi_addr = SocketAddrV4::new(config.multicast_address, config.multicast_port);
    let std_socket = bind_multicast(&addr, &multi_addr)?;
    let udp_socket = UdpSocket::from_std(std_socket)?;

    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    tokio::spawn(async move {
        while let Some(command) = rx.recv().await {
            println!("sending message");
            match udp_socket
                .send_to(
                    serde_json::to_string(&command).unwrap().as_bytes(),
                    multi_addr,
                )
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("failed to send");
                    eprintln!("{}", e);
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

                println!("read {} bytes from {}", n, addr);

                let st = std::str::from_utf8(&buf[..n]).unwrap();

                println!("{}", st);

                let command: Command = serde_json::from_str(st).unwrap();

                dbg!(&command);

                tx.send(command).await.unwrap();
            }
        });
    }
}

/// Bind socket to multicast address with IP_MULTICAST_LOOP and SO_REUSEADDR Enabled
fn bind_multicast(
    addr: &SocketAddrV4,
    multi_addr: &SocketAddrV4,
) -> Result<std::net::UdpSocket, Box<dyn std::error::Error>> {
    assert!(multi_addr.ip().is_multicast(), "Must be multcast address");

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
        multicast_address: std::env::var("MULTICAST_ADDRESS")
            .unwrap_or(DEFAULT_MULTICAST_ADDRESS.to_string())
            .parse()?,
        multicast_port: std::env::var("MULTICAST_PORT")
            .unwrap_or(DEFAULT_MULTICAST_PORT.to_string())
            .parse()?,
    })
}
