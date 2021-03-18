use std::net::SocketAddr;

use async_stream::stream;
use futures::{SinkExt, Stream, StreamExt};
use matchbook_types::*;
use std::collections::HashMap;

pub fn message_sequencer_stream<S: Stream<Item = (Message, SocketAddr)> + Unpin>(
    mut stream: S,
    buf_size: usize,
    // need to add param to re-request messages
) -> impl Stream<Item = (Message, SocketAddr)> {
    stream! {
        let mut next_expected_id = 0;
        let mut cursor = 0;
        let mut buf: Vec<Option<Message>> = vec![None; buf_size];

        while let Some((message, addr)) = stream.next().await {
            let id = message.id.topic_sequence_n as usize;

            if id == next_expected_id {
                yield (message, addr);

                next_expected_id += 1;

                // emit any cached messages
                while let Some(message) = buf[cursor].clone() {
                    buf[cursor] = None;
                    cursor = (cursor + 1) % buf.len();
                    next_expected_id += 1;
                    yield (message, addr);
                }
            }
            // received element out of order
            else if id > next_expected_id {
                let offset = id - next_expected_id - 1;
                let write_idx = (offset + cursor) % buf.len();
                buf[write_idx] = Some(message.clone());

                // TODO(will): need to re-request missing elements
                // TODO(will): need to re-allocate if the buffer is going to grow too large

            }
         }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn sequencer_stream_writes_outputs_in_correct_order_and_removes_duplicates() {
        let (mut tx, rx) = futures::channel::mpsc::channel(8);

        let stream = message_sequencer_stream(rx, 2);

        let socket_addr = SocketAddr::new([0, 0, 0, 0].into(), 12345);

        let messages: Vec<_> = (0..5)
            .map(|x| {
                (
                    Message {
                        id: MessageId {
                            publisher_id: ServiceId {
                                kind: ServiceKind::Port,
                                number: 0,
                            },
                            topic_id: "client1".to_string(),
                            topic_sequence_n: x,
                        },
                        kind: MessageKind::LimitOrderSubmitRequest {
                            price: 100,
                            quantity: 100,
                            symbol: ['A', 'D', 'B', 'E'],
                            side: Side::Ask,
                        },
                    },
                    socket_addr,
                )
            })
            .collect();

        let sending_order: &[usize] = &[0, 3, 2, 4, 1, 2];

        for &i in sending_order {
            tx.send(messages[i].clone()).await.unwrap();
        }

        drop(tx); // drop tx to close the channel

        futures::pin_mut!(stream);

        let sampled: Vec<_> = stream.take(5).collect().await;

        assert_eq!(messages, sampled);

        let messages: Vec<_> = (5..12)
            .map(|x| {
                (
                    Message {
                        id: MessageId {
                            publisher_id: ServiceId {
                                kind: ServiceKind::Port,
                                number: 0,
                            },
                            topic_id: "client1".to_string(),
                            topic_sequence_n: x,
                        },
                        kind: MessageKind::LimitOrderSubmitRequest {
                            price: 100,
                            quantity: 100,
                            symbol: ['A', 'D', 'B', 'E'],
                            side: Side::Ask,
                        },
                    },
                    socket_addr,
                )
            })
            .collect();
    }
}
