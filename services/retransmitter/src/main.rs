#![deny(clippy::all)]
use {
    futures::{SinkExt, StreamExt},
    matchbook_types::*,
    matchbook_util::*,
    std::{
        collections::HashMap,
        net::{SocketAddr, SocketAddrV4},
    },
    tokio::net::UdpSocket,
    tokio_util::udp::UdpFramed,
    tracing::*,
};

const DEFAULT_MULTICAST_ADDRESS: [u8; 4] = [239, 255, 42, 98];
const DEFAULT_MULTICAST_PORT: u16 = 50692;
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let multicast_addr = SocketAddr::new(DEFAULT_MULTICAST_ADDRESS.into(), DEFAULT_MULTICAST_PORT);

    let udp_socket = {
        let socket = bind_multicast(
            &SocketAddrV4::new(IP_ALL.into(), DEFAULT_MULTICAST_PORT.into()),
            &SocketAddrV4::new(DEFAULT_MULTICAST_ADDRESS.into(), DEFAULT_MULTICAST_PORT),
        )?;

        let socket = UdpSocket::from_std(socket)?;

        let socket = UdpFramed::new(socket, MatchbookMessageCodec::new());
        socket
    };

    info!("started listening");

    let (mut sink, mut stream) = udp_socket.split();
    let mut cache = HashMap::new();

    loop {
        let message = stream.next().await;

        match message {
            Some(Ok((message, _))) => {
                debug!(?message.id, "received message");

                match message.kind {
                    MessageKind::RetransmitRequest => {
                        if let Some(msg) = cache.get(&message.id).cloned() {
                            sink.send((msg, multicast_addr)).await?;
                        }
                    }
                    // we don't want to cache any retransmit requests, or any message that we've already seen
                    _ => {
                        if !cache.contains_key(&message.id) {
                            cache.insert(message.id.clone(), message.clone());
                        }
                    }
                }
            }
            _ => continue,
        }
    }

    Ok(())
}
