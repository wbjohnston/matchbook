use env_logger;
use log;
use matchbook_types::*;
use matchbook_util::*;
use matching_engine::*;
use serde_json;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use tokio::net::UdpSocket;

const DEFAULT_MULTICAST_ADDRESS: &'static str = "239.255.42.98";
const DEFAULT_MULTICAST_PORT: &'static str = "50692";
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
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
        log::info!("received {} bytes from client at {}", n, addr);

        let message_as_str = match std::str::from_utf8(&buf[..n]) {
            Ok(s) => s,
            Err(e) => {
                log::error!("failed to parse received message into utf8. ignoring message");
                log::error!("{}", e);
                continue;
            }
        };

        let command: Command = match serde_json::from_str(message_as_str) {
            Ok(command) => command,
            Err(e) => {
                log::error!("failed to parse message into command");
                log::error!("{}", e);
                continue;
            }
        };

        match command {
            Command::LimitOrderSubmitRequest {
                quantity,
                price,
                symbol,
                side,
            } => {
                // FIXME:
                log::info!(
                    "received request to open {:?} limit order for {} {} at {}",
                    side,
                    quantity,
                    symbol,
                    price
                );
                let fills = engine.submit_limit_order(side, symbol.as_str(), price, quantity)?;
                fills.iter().for_each(|execution| {
                    log::info!(
                        "executed {} shares of {} at {}",
                        execution.quantity,
                        symbol,
                        execution.price
                    )
                });
            }
        }
    }
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
