use {
    crate::{message, Context, ParticipantChannelMap},
    fixer_upper::{Header as FixHeader, Message as FixMessage, MessageType as FixMessageType},
    futures::{Sink, SinkExt, Stream, StreamExt},
    matchbook_types::*,
    matchbook_util::*,
    std::net::SocketAddr,
    tokio::{
        net::TcpListener,
        sync::mpsc::{Receiver, Sender},
    },
    tracing::*,
};

pub async fn spawn_listen_handler<'a>(
    listener: TcpListener,
    udp_tx: Sender<(Message, SocketAddr)>,
    state: ParticipantChannelMap,
    context: Context,
) {
    info!("started listening on {}", listener.local_addr().unwrap());
    while let Ok((stream, addr)) = listener.accept().await {
        let span = debug_span!("connection accepted");
        let _enter = span.enter();

        info!("accepted connection from {}", addr);
        let udp_tx = udp_tx.clone();
        let state = state.clone();
        let context = context.clone();

        tokio::spawn(
            async move { spawn_client_handler(stream, udp_tx, addr, state, context).await },
        );
    }
}

pub async fn spawn_client_handler(
    stream: tokio::net::TcpStream,
    udp_tx: Sender<(Message, SocketAddr)>,
    addr: SocketAddr,
    state: ParticipantChannelMap,
    context: Context,
) {
    let (participant_tx, mut participant_rx): (Sender<(Message, SocketAddr)>, _) =
        tokio::sync::mpsc::channel(32);
    let framed = tokio_util::codec::Framed::new(stream, FixJsonCodec::new());
    let (mut sink, mut stream) = framed.split();

    // FIX session sequence numbers start at 1
    let mut inbound_sequence_n = 1;
    let mut outbound_sequence_n = 1;

    // create a channel that will be used to start listening for UDP messages after the user has
    let (sender_tx, mut sender_rx) = tokio::sync::mpsc::channel(32);
    let (logon_tx, logon_rx) = tokio::sync::oneshot::channel();
    let listen_handle = {
        let context = context.clone();
        tokio::spawn(async move {
            let udp_tx = udp_tx.clone();
            let participant_id = loop {
                let span = debug_span!("client message received", ?addr);
                let _enter = span.enter();

                let message = stream.next().await;

                if let Some(Ok(ref message)) = message {
                    let sequence_n = message.header.msg_seq_num;
                    if sequence_n != inbound_sequence_n {
                        error!(
                        "received unexpected sequence number {}, was expecting {}. terminating connection",
                        sequence_n, inbound_sequence_n
                    );
                        sender_tx
                            .send(fixer_upper::Message {
                                header: fixer_upper::Header {
                                    begin_string: fixer_upper::BeginString::Fix_4_4,
                                    body_length: None,
                                    msg_type: fixer_upper::MessageType::Logout,
                                    sender_comp_id: message.header.target_comp_id.clone(),
                                    target_comp_id: context.exchange_id.clone(),
                                    msg_seq_num: outbound_sequence_n,
                                    sending_time: chrono::Utc::now(),
                                },
                                body: fixer_upper::Body {
                                    ..fixer_upper::Body::default()
                                },
                                trailer: fixer_upper::Trailer {
                                    signature: None,
                                    signature_length: None,
                                },
                            })
                            .await
                            .unwrap();

                        return;
                    }
                }

                inbound_sequence_n += 1;

                match message {
                    Some(Ok(FixMessage {
                        header:
                            FixHeader {
                                ref sender_comp_id,
                                msg_type: FixMessageType::Logon,
                                ..
                            },
                        ..
                    })) => {
                        let id = sender_comp_id.clone();
                        let mut message = message.unwrap().unwrap();
                        message.header.sender_comp_id = message.header.target_comp_id;
                        message.header.target_comp_id = id.clone();
                        sender_tx.send(message).await.unwrap();
                        break id;
                    }
                    None => return,
                    x => {
                        warn!(?x, "received non-logon message");
                        continue;
                    }
                }
            };

            logon_tx.send(participant_id).unwrap();

            // normal flow
            loop {
                let message = stream.next().await;
                // TODO(will): check if sequence number matches
                if let Some(Ok(ref message)) = message {
                    let sequence_n = message.header.msg_seq_num;
                    if sequence_n != inbound_sequence_n {
                        error!(
                        "received unexected sequence number {}, was expecting {}. terminating connection",
                        sequence_n, inbound_sequence_n
                    );

                        sender_tx
                            .send(fixer_upper::Message {
                                header: fixer_upper::Header {
                                    begin_string: fixer_upper::BeginString::Fix_4_4,
                                    body_length: None,
                                    msg_type: fixer_upper::MessageType::Logout,
                                    sender_comp_id: message.header.target_comp_id.clone(),
                                    target_comp_id: "matchbook".to_string(),
                                    msg_seq_num: outbound_sequence_n,
                                    sending_time: chrono::Utc::now(),
                                },
                                body: fixer_upper::Body {
                                    ..fixer_upper::Body::default()
                                },
                                trailer: fixer_upper::Trailer {
                                    signature: None,
                                    signature_length: None,
                                },
                            })
                            .await
                            .unwrap();

                        return;
                    }
                }

                inbound_sequence_n += 1;

                match message {
                    Some(Ok(message)) => {
                        let message = match message::fix_message_into_matchbook_message(
                            message,
                            context.service_id,
                        ) {
                            Ok(message) => message,
                            Err(e) => {
                                warn!("{}", e);
                                continue;
                            }
                        };

                        udp_tx
                            .send((message, addr))
                            .await
                            .expect("failed the send message to backbone transmitter");
                    }
                    None => break,
                    x => warn!("{:?}", x),
                }
            }
        })
    };

    let sender_handle = {
        let context = context.clone();
        tokio::spawn(async move {
            // TODO(will): need to listen for messages coming from listener task
            //              specifically, need to be able to echo back messages like Logon that
            //              dont need to be sent to other services
            loop {
                tokio::select! {
                    Some((message, _)) = participant_rx.recv()=> {
                        let mut message = message::matchbook_message_into_fix_message(message, context.exchange_id.clone());
                        message.header.msg_seq_num = outbound_sequence_n;
                        sink.send(message)
                            .await
                            .expect("failed to write message to client");
                        outbound_sequence_n += 1;
                    }
                    Some(message) = sender_rx.recv() => {
                        sink.send(message).await.unwrap()
                    }
                    else => {
                        break
                    }
                }
            }
        })
    };

    let participant_id = match logon_rx.await {
        Ok(id) => id,
        Err(_) => {
            warn!("user terminated connection before logging on");
            return;
        }
    };

    state
        .write()
        .await
        .insert(participant_id.clone(), participant_tx);
    info!("user authenticated and ready to receive messages");

    tokio::select!(_ = listen_handle => { }, _ = sender_handle => { });
    info!("ending session for participant {}", participant_id);
}

pub async fn spawn_multicast_rx_handler<S>(
    mut stream: S,
    state: ParticipantChannelMap,
    _context: Context,
) where
    S: Stream<Item = Result<(Message, SocketAddr), std::io::Error>> + Unpin + std::fmt::Debug,
{
    while let Some(Ok((message, addr))) = stream.next().await {
        if let Some(tx) = state.read().await.get(&message.id.topic_id).cloned() {
            debug!("received message",);
            tx.send((message, addr))
                .await
                .expect("failed to send to backbone");
            trace!("message forwarded to client connection handler");
        } else {
            warn!("message meant for invalid participant");
        }
    }
}

pub async fn spawn_multicast_tx_handler<S>(
    mut sink: S,
    mut rx: Receiver<(Message, SocketAddr)>,
    context: Context,
) where
    S: Sink<(Message, SocketAddr), Error = Box<dyn std::error::Error>> + Unpin + std::fmt::Debug,
{
    let multicast_addr = context.multicast_addr;
    while let Some((message, _addr)) = rx.recv().await {
        debug!(?message.id, "received message");
        sink.send((message, multicast_addr))
            .await
            .expect("failed to send to backbone");
        trace!("message sent to client");
    }
}
