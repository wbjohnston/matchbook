use error::Error;
use futures::{SinkExt, StreamExt};
use matchbook_types::*;
use matchbook_util::{bind_multicast, MatchbookCodec};
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::{collections::HashMap, error};
use tokio::net::TcpListener;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use tokio_util::udp::UdpFramed;
use tracing::*;

const DEFAULT_MULTICAST_ADDRESS: [u8; 4] = [239, 255, 42, 98];
const DEFAULT_MULTICAST_PORT: u16 = 50692;
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

type ParticipantChannelMap = Arc<RwLock<HashMap<ParticipantId, Sender<(Message, SocketAddr)>>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let state: ParticipantChannelMap = Arc::new(RwLock::new(HashMap::new()));
    let udp_socket = {
        let socket = bind_multicast(
            &SocketAddrV4::new(IP_ALL.into(), DEFAULT_MULTICAST_PORT.into()),
            &SocketAddrV4::new(DEFAULT_MULTICAST_ADDRESS.into(), DEFAULT_MULTICAST_PORT),
        )?;

        let socket = UdpSocket::from_std(socket)?;

        let socket = UdpFramed::new(socket, MatchbookCodec::new());
        socket
    };

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    let (udp_tx, mut udp_rx) = tokio::sync::mpsc::channel(32);

    let (mut sink, mut stream) = udp_socket.split();

    // A task responsible for handling incoming client connections
    let client_listener_handle = {
        let state = state.clone();
        let udp_tx = udp_tx.clone();
        tokio::spawn(async move {
            info!("started listening on {}", listener.local_addr().unwrap());
            // every time someone new connects, we assign them a new ID. In the future this will be replaced by a session manager
            let mut participant_id_counter = 0;
            let state = state.clone();
            while let Ok((stream, addr)) = listener.accept().await {
                let span = span!(Level::DEBUG, "client_connection", ?addr);
                let _enter = span.enter();
                info!("accepted connection");

                let udp_tx = udp_tx.clone();
                let state = state.clone();
                let participant_id = participant_id_counter;
                participant_id_counter += 1;

                let (tx, mut rx) = tokio::sync::mpsc::channel(32);
                state.write().await.insert(participant_id, tx);
                tokio::spawn(async move {
                    let framed = tokio_util::codec::Framed::new(stream, MatchbookCodec::new());
                    let (mut sink, mut stream) = framed.split();

                    let listen_handle = tokio::spawn(async move {
                        let udp_tx = udp_tx.clone();
                        loop {
                            match stream.next().await {
                                Some(Ok(mut message)) => {
                                    message.topic_id = participant_id;

                                    udp_tx
                                        .send((message, addr))
                                        .await
                                        .expect("failed the send message to backbone transmitter");
                                }
                                None => break,
                                x => warn!("{:?}", x),
                            }
                        }
                    });

                    let sender_handle = tokio::spawn(async move {
                        while let Some((message, _addr)) = rx.recv().await {
                            sink.send(message)
                                .await
                                .expect("failed to write message to client")
                        }
                    });

                    tokio::select! {
                        _ = listen_handle => {},
                        _ = sender_handle => {}
                    }
                });
            }
        })
    };

    //A task that listens for inbound multicast packets and forwards them to the applicable client handler
    let multicast_rx_handle = tokio::spawn(async move {
        let state = state.clone();
        while let Some(Ok((message, addr))) = stream.next().await {
            let span = span!(Level::DEBUG, "udp_listener", message.topic_id);
            let _enter = span.enter();
            if let Some(tx) = state.read().await.get(&message.topic_id).cloned() {
                debug!("received message",);
                tx.send((message, addr))
                    .await
                    .expect("failed to send to backbone");
                trace!("message forwarded to client connection handler");
            } else {
                warn!("message meant for invalid participant");
            }
        }
    });

    // A task that waits for messages from clients to be published via multicast
    let multicast_tx_handle = tokio::spawn(async move {
        let span = debug_span!("udp_sender");
        let _handle = span.enter();
        while let Some((message, addr)) = udp_rx.recv().await {
            debug!(addr = ?addr, participant_id = message.topic_id, "received message");
            let multicast_addr =
                SocketAddr::new(DEFAULT_MULTICAST_ADDRESS.into(), DEFAULT_MULTICAST_PORT);
            sink.send((message, multicast_addr))
                .await
                .expect("failed to send to backbone");
            trace!("message sent to client");
        }
    });

    tokio::try_join!(
        client_listener_handle,
        multicast_rx_handle,
        multicast_tx_handle
    )?;

    Ok(())
}
