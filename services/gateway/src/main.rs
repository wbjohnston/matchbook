use tokio::net::{TcpListener, UdpSocket};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::{env, net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4}};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let port: u16 = env::var("PORT")?.parse()?;
    println!("starting listening on port {}", port);
    let tcp_listener_addr: SocketAddrV4 = format!("0.0.0.0:{}", port).parse()?;
    let tcp_listener = TcpListener::bind(tcp_listener_addr).await?;
    println!("started listening at address = {}", tcp_listener_addr);

    println!("starting udp multicast emmitter at {}", 1234);
    let udp_tx_addr = Ipv4Addr::new(224, 0, 0, 123);
    assert!(udp_tx_addr.is_multicast());
    // let udp_emitter = UdpSocket::bind(udp_tx_addr).await?;
    println!("started udp multicast emitter at {}", 1234);

    loop {
        let (mut socket, _) = tcp_listener.accept().await?;
        tokio::spawn(async move {
            println!("accepted connection");
            let mut buf = [0; 1024];

            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    _ => unimplemented!()
                };

                // Write the data back
                // if let Err(e) = udp_emitter.send(&buf[0..n]).await {
                //     eprintln!("failed to write to socket; err = {:?}", e);
                //     return;
                // }
            }

        });
    }
}
