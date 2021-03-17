#![deny(clippy::all)]

use {
    futures::{Sink, SinkExt, Stream, StreamExt},
    matchbook_types::*,
    matchbook_util::*,
    std::{
        collections::HashMap,
        marker::Unpin,
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
            &SocketAddrV4::new(IP_ALL.into(), DEFAULT_MULTICAST_PORT),
            &SocketAddrV4::new(DEFAULT_MULTICAST_ADDRESS.into(), DEFAULT_MULTICAST_PORT),
        )?;

        let socket = UdpSocket::from_std(socket)?;
        UdpFramed::new(socket, MatchbookMessageCodec::new())
    };

    info!("started listening");

    let (sink, stream) = udp_socket.split();

    spawn_main_loop(stream, sink, multicast_addr).await;
    Ok(())
}

pub async fn spawn_main_loop<St, Si>(mut stream: St, mut sink: Si, multicast_addr: SocketAddr)
where
    St: Stream<Item = Result<(Message, SocketAddr), std::io::Error>> + Unpin,
    Si: Sink<(Message, SocketAddr)> + Unpin,
{
    let mut cache = HashMap::new();
    loop {
        let message = stream.next().await;

        match message {
            Some(Ok((message, _))) => {
                debug!(?message.id, "received message");

                match message.kind {
                    MessageKind::RetransmitRequest => {
                        if let Some(msg) = cache.get(&message.id).cloned() {
                            let _ = sink.send((msg, multicast_addr)).await;
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
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::SinkExt;
    use futures_test::*;

    #[tokio::test]
    async fn retransmits_first_message_received_when_a_message_with_a_duplicate_id_is_received() {
        let (sink_tx, mut sink_rx) = futures::channel::mpsc::channel(1);
        let (mut stream_tx, stream_rx) = futures::channel::mpsc::channel(1);

        let multicast_addr =
            SocketAddr::new(DEFAULT_MULTICAST_ADDRESS.into(), DEFAULT_MULTICAST_PORT);

        tokio::spawn(async move { spawn_main_loop(stream_rx, sink_tx, multicast_addr).await });

        let id = MessageId {
            publisher_id: ServiceId {
                kind: ServiceKind::Port,
                number: 0,
            },
            topic_id: "client1".to_owned(),
            topic_sequence_n: 0,
        };

        let first_received = Message {
            id: id.clone(),
            sending_time: chrono::Utc::now(),
            kind: MessageKind::LimitOrderSubmitRequest {
                side: Side::Bid,
                price: 100,
                quantity: 100,
                symbol: ['A', 'D', 'B', 'E'],
            },
        };

        let id_colliding_message = Message {
            id: id.clone(),
            sending_time: chrono::Utc::now(),
            kind: MessageKind::LimitOrderSubmitRequest {
                side: Side::Bid,
                price: 100,
                quantity: 100,
                symbol: ['A', 'D', 'B', 'E'],
            },
        };

        let retransmit_req = Message {
            id: id.clone(),
            kind: MessageKind::RetransmitRequest,
            sending_time: chrono::Utc::now(),
        };

        stream_tx
            .send(Ok((first_received.clone(), multicast_addr)))
            .await
            .unwrap();

        assert_stream_pending!(sink_rx);

        stream_tx
            .send(Ok((id_colliding_message, multicast_addr)))
            .await
            .unwrap();

        assert_stream_pending!(sink_rx);

        stream_tx
            .send(Ok((retransmit_req.clone(), multicast_addr)))
            .await
            .unwrap();

        assert_eq!(
            sink_rx.next().await.unwrap(),
            (first_received.clone(), multicast_addr),
            "received different message than the first message the retransmitter received"
        );
        assert_stream_pending!(sink_rx);

        stream_tx
            .send(Ok((retransmit_req.clone(), multicast_addr)))
            .await
            .unwrap();

        assert_eq!(
            sink_rx.next().await.unwrap(),
            (first_received.clone(), multicast_addr),
            "received different message after requesting another retranimst. the same message should always be retransmitted"
        );
        assert_stream_pending!(sink_rx);
    }

    #[tokio::test]
    async fn doesnt_retransmit_an_unknown_message() {
        let (sink_tx, mut sink_rx) = futures::channel::mpsc::channel(1);
        let (mut stream_tx, stream_rx) = futures::channel::mpsc::channel(1);

        let multicast_addr =
            SocketAddr::new(DEFAULT_MULTICAST_ADDRESS.into(), DEFAULT_MULTICAST_PORT);

        tokio::spawn(async move { spawn_main_loop(stream_rx, sink_tx, multicast_addr).await });

        stream_tx
            .send(Ok((
                Message {
                    id: MessageId {
                        publisher_id: ServiceId {
                            kind: ServiceKind::Port,
                            number: 0,
                        },
                        topic_id: String::from("foobar"),
                        topic_sequence_n: 1000,
                    },
                    sending_time: chrono::Utc::now(),
                    kind: MessageKind::RetransmitRequest,
                },
                multicast_addr,
            )))
            .await
            .unwrap();

        assert_stream_pending!(sink_rx);
    }

    #[tokio::test]
    async fn retransmits_messages_in_the_order_retransmit_requests_arrive() {
        let to_retransmit_id_1 = MessageId {
            publisher_id: ServiceId {
                kind: ServiceKind::Port,
                number: 0,
            },
            topic_id: "client1".to_owned(),
            topic_sequence_n: 0,
        };

        let to_retransmit_1 = Message {
            id: to_retransmit_id_1.clone(),
            sending_time: chrono::Utc::now(),
            kind: MessageKind::LimitOrderSubmitRequest {
                side: Side::Bid,
                price: 100,
                quantity: 100,
                symbol: ['A', 'D', 'B', 'E'],
            },
        };

        let to_retransmit_id_2 = MessageId {
            publisher_id: ServiceId {
                kind: ServiceKind::Port,
                number: 0,
            },
            topic_id: "client2".to_owned(),
            topic_sequence_n: 0,
        };

        let to_retransmit_2 = Message {
            id: to_retransmit_id_2.clone(),
            sending_time: chrono::Utc::now(),
            kind: MessageKind::LimitOrderSubmitRequest {
                side: Side::Bid,
                price: 100,
                quantity: 100,
                symbol: ['A', 'D', 'B', 'E'],
            },
        };

        let retransmit_1_req = Message {
            id: to_retransmit_id_1.clone(),
            kind: MessageKind::RetransmitRequest,
            sending_time: chrono::Utc::now(),
        };

        let retransmit_2_req = Message {
            id: to_retransmit_id_2.clone(),
            kind: MessageKind::RetransmitRequest,
            sending_time: chrono::Utc::now(),
        };

        let (sink_tx, mut sink_rx) = futures::channel::mpsc::channel(1);
        let (mut stream_tx, stream_rx) = futures::channel::mpsc::channel(1);

        let multicast_addr =
            SocketAddr::new(DEFAULT_MULTICAST_ADDRESS.into(), DEFAULT_MULTICAST_PORT);

        tokio::spawn(async move { spawn_main_loop(stream_rx, sink_tx, multicast_addr).await });

        // send messages for retrieval
        stream_tx
            .send(Ok((to_retransmit_1.clone(), multicast_addr)))
            .await
            .unwrap();

        stream_tx
            .send(Ok((to_retransmit_2.clone(), multicast_addr)))
            .await
            .unwrap();

        assert_stream_pending!(sink_rx);

        // request to retransmit
        stream_tx
            .send(Ok((retransmit_2_req, multicast_addr)))
            .await
            .unwrap();

        stream_tx
            .send(Ok((retransmit_1_req, multicast_addr)))
            .await
            .unwrap();

        assert_eq!(
            sink_rx.next().await.unwrap(),
            (to_retransmit_2, multicast_addr)
        );
        assert_eq!(
            sink_rx.next().await.unwrap(),
            (to_retransmit_1, multicast_addr)
        );
    }
}
