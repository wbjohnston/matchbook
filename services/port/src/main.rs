use env_logger;
use log;
use matchbook_types::*;
use matchbook_util::*;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, UdpSocket};

const DEFAULT_MULTICAST_ADDRESS: &'static str = "239.255.42.98";
const DEFAULT_MULTICAST_PORT: &'static str = "50692";
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let config = get_config_from_env()?;
    let tcp_listener_addr: SocketAddrV4 = format!("0.0.0.0:{}", 8080).parse()?;
    let tcp_listener = TcpListener::bind(tcp_listener_addr).await?;
    log::info!("started listening for TCP clients on {}", tcp_listener_addr);

    let addr = SocketAddrV4::new(IP_ALL.into(), config.multicast_port);
    let multi_addr = SocketAddrV4::new(config.multicast_address, config.multicast_port);
    let std_socket = bind_multicast(&addr, &multi_addr)?;
    let udp_socket = UdpSocket::from_std(std_socket)?;
    let udp_socket = std::sync::Arc::new(udp_socket);
    log::info!("bound to multicast on {}", udp_socket.local_addr()?);

    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    {
        let udp_socket = udp_socket.clone();
        // UDP listener
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            while let Ok((n, addr)) = udp_socket.recv_from(&mut buf).await {
                log::info!("READER received {} bytes from {}", n, addr);

                let s = std::str::from_utf8(&buf).unwrap();
                let message: Message = serde_json::from_str(&s[..n]).unwrap();
                dbg!(&message);
            }
        });
    }

    // UDP transmitter
    {
        let udp_socket = udp_socket.clone();
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
    }

    let mut participant_id_map: HashMap<IpAddr, u64> = HashMap::new();
    let mut participant_counter = 0;

    loop {
        let (mut socket, addr) = tcp_listener.accept().await?;
        // TODO(will): this participant id generation needs to be smarter and will eventually
        //              need to associate participant ids with account ids
        let participant_id = if let Some(id) = participant_id_map.get(&addr.ip()) {
            id.clone()
        } else {
            let participant_id = participant_counter;
            participant_counter += 1;
            participant_id_map.insert(addr.ip(), participant_id);
            participant_id
        };

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
                // TODO(will): this needs to be replaced with message type specific to clients.
                //              In the long run, these will be FIX messages. But in the short term they'll need to be some other format
                let command: MessageKind = serde_json::from_str(st).unwrap();
                {
                    match &command {
                        MessageKind::LimitOrderSubmitRequest {
                            symbol,
                            price,
                            quantity,
                            side,
                        } => {
                            log::info!(
                                "received limit order submit request to {:?} {} {} at {} from client at {} with participant id {}",
                                side,
                                symbol,
                                quantity,
                                price,
                                addr,
                                participant_id
                            );
                        }
                        _ => continue,
                    }
                }

                let message = Message {
                    participant_id: participant_id,
                    service_id: config.service_identifier,
                    kind: command,
                };

                match tx.send(message).await {
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

#[derive(Debug, Clone, Copy)]
struct Config {
    pub service_identifier: ServiceId,
    pub multicast_address: Ipv4Addr,
    pub multicast_port: u16,
}

fn get_config_from_env() -> Result<Config, Box<dyn std::error::Error>> {
    Ok(Config {
        service_identifier: std::env::var("SERVICE_IDENTIFIER")
            .map(|x| ServiceId::from_str(x.as_str()).unwrap())
            .expect("missing required environment variable 'SERVICE_IDENTIFIER'"),

        multicast_address: std::env::var("MULTICAST_ADDRESS")
            .unwrap_or(DEFAULT_MULTICAST_ADDRESS.to_string())
            .parse()?,

        multicast_port: std::env::var("MULTICAST_PORT")
            .unwrap_or(DEFAULT_MULTICAST_PORT.to_string())
            .parse()?,
    })
}
