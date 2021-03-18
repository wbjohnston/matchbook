#![deny(clippy::all)]

use std::io::BufReader;

use tokio_rustls::{
    rustls::{
        internal::pemfile::{certs, pkcs8_private_keys},
        NoClientAuth, ServerConfig,
    },
    TlsAcceptor,
};
mod config;
mod handler;
mod message;

use {
    futures::StreamExt,
    handler::*,
    matchbook_types::*,
    matchbook_util::*,
    std::{
        collections::HashMap,
        error::Error,
        net::{SocketAddr},
        sync::Arc,
    },
    tokio::{
        net::{TcpListener, UdpSocket},
        sync::{mpsc::Sender, RwLock},
    },
    tokio_util::udp::UdpFramed,
};

const IP_ALL: [u8; 4] = [0, 0, 0, 0];

pub type ParticipantChannelMap = Arc<RwLock<HashMap<ParticipantId, Sender<(Message, SocketAddr)>>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = config::source_config_from_env()?;
    tracing_subscriber::fmt::init();
    let context = Context {
        exchange_id: config.exchange_id,
        service_id: config.service_id,
        multicast_addr: config.multicast_addr,
    };

    let state: ParticipantChannelMap = Arc::new(RwLock::new(HashMap::new()));
    let udp_socket = {
        let socket = bind_multicast(
            &SocketAddr::new(IP_ALL.into(), config.multicast_addr.port()),
            &config.multicast_addr,
        )?;

        let socket = UdpSocket::from_std(socket)?;

        UdpFramed::new(socket, MatchbookMessageCodec::new())
    };

    let (udp_tx, udp_rx) = tokio::sync::mpsc::channel(32);

    let (sink, stream) = udp_socket.split();

    let mut tls_config = ServerConfig::new(NoClientAuth::new());
    let certs = certs(&mut BufReader::new(std::io::Cursor::new(config.tls_cert))).unwrap();
    let keys = pkcs8_private_keys(&mut BufReader::new(std::io::Cursor::new(
        config.tls_cert_key,
    )))
    .unwrap();

    tls_config.set_single_cert(certs, keys[0].clone())?;

    let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    // A task responsible for handling incoming client connections
    let client_listener_handle = {
        let state = state.clone();
        let context = context.clone();
        tokio::spawn(async move {
            spawn_listen_handler(listener, tls_acceptor, udp_tx, state.clone(), context).await
        })
    };

    //A task that listens for inbound multicast packets and forwards them to the applicable client handler
    let multicast_rx_handle = {
        let state = state.clone();
        let context = context.clone();
        tokio::spawn(async move { spawn_multicast_rx_handler(stream, state, context).await })
    };

    let multicast_tx_handle = {
        let context = context.clone();
        tokio::spawn(async move { spawn_multicast_tx_handler(sink, udp_rx, context).await })
    };

    let _ = tokio::join!(
        client_listener_handle,
        multicast_rx_handle,
        multicast_tx_handle
    );

    Ok(())
}

#[derive(Debug, Clone)]
pub struct Context {
    pub service_id: ServiceId,
    pub exchange_id: String,
    pub multicast_addr: SocketAddr,
}
