use async_stream::stream;
use futures::{SinkExt, Stream, StreamExt};
use matchbook_types::*;
use std::collections::HashMap;

pub fn message_sequencer_stream(
    mut stream: impl futures::Stream<Item = Result<Message, std::io::Error>> + Unpin,
    mut rerequest_tx: tokio::sync::mpsc::Sender<(String, u64)>,
    buf_size: usize,
    // need to add param to re-request messages
) -> impl Stream<Item = Result<Message, std::io::Error>> {
    stream! {
        let mut topics = HashMap::new();

        struct TopicBuffer {
            buf: Vec<Option<Message>>,
            next_expected_id: usize,
            cursor: usize
        }

        while let Some(Ok(message)) = stream.next().await {
            let topic = message.id.topic_id.clone();
            let id = message.id.topic_sequence_n as usize;

            let entry = topics.entry(topic.clone()).or_insert(TopicBuffer {
                buf: vec![None; buf_size],
                next_expected_id: 0,
                cursor: 0,
            });

            if id == entry.next_expected_id {
                yield Ok(message);

                entry.next_expected_id += 1;

                // emit any cached messages
                loop {
                    if let Some(msg) = entry.buf[entry.cursor].take() {
                        entry.cursor = (entry.cursor + 1) % entry.buf.len();
                        entry.next_expected_id += 1;
                        yield Ok(msg);
                    } else {
                        break
                    }
                }
            }
            // received element out of order
            else if id > entry.next_expected_id {
                let offset = id - entry.next_expected_id - 1;

                // if we are going to have an entry that may overwrite an element in our buffer, we need to resize
                let should_resize_buffer = offset > (entry.buf.len() - 1);
                if should_resize_buffer {
                    let mut new_buf = vec![None; entry.buf.len() * 2];
                    // FIXME(will): this causes unnecessary copies
                    for (dst, x) in entry.buf.iter().cloned().cycle().skip(entry.cursor).take(entry.buf.len()).enumerate() {
                        new_buf[dst] = x;
                    }
                    entry.buf = new_buf;
                    entry.cursor = 0;
                }
                let write_idx = (offset + entry.cursor) % entry.buf.len();
                entry.buf[write_idx] = Some(message.clone());

                // re-request everything that we're missing
                let mut id_offset = 1;
                let mut scan_cursor = entry.cursor;

                let _ = rerequest_tx.send((topic.clone(), entry.next_expected_id as u64)).await;
                while scan_cursor != write_idx {
                    if entry.buf[scan_cursor].is_none() {
                        let id_to_rerequest = entry.next_expected_id + id_offset;
                        let _ = rerequest_tx.send((topic.clone(), id_to_rerequest as u64)).await;
                    }

                    id_offset += 1;
                    scan_cursor = (scan_cursor + 1) % entry.buf.len();
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use futures::SinkExt;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn sequencer_stream_writes_outputs_in_correct_order_and_removes_duplicates() {
        let (mut tx, rx) = futures::channel::mpsc::unbounded();
        let (r_tx, mut r_rx) = tokio::sync::mpsc::channel(32);

        let stream = message_sequencer_stream(rx, r_tx, 1);

        let ordered_client1_messages: Vec<_> = (0..3)
            .map(|x| Message {
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
            })
            .collect();

        let ordered_client2_messages: Vec<_> = (0..3)
            .map(|x| Message {
                id: MessageId {
                    publisher_id: ServiceId {
                        kind: ServiceKind::Port,
                        number: 0,
                    },
                    topic_id: "client2".to_string(),
                    topic_sequence_n: x,
                },
                kind: MessageKind::LimitOrderSubmitRequest {
                    price: 100,
                    quantity: 100,
                    symbol: ['A', 'D', 'B', 'E'],
                    side: Side::Ask,
                },
            })
            .collect();

        let mut messages: Vec<_> = ordered_client1_messages.clone();
        messages.extend(ordered_client2_messages.clone());

        let sending_order = vec![0, 5, 0, 4, 3, 2, 1, 3, 4];

        for i in sending_order {
            tx.send(Ok(messages[i].clone())).await.unwrap();
        }

        drop(tx); // drop tx to close the channel

        let y = tokio::spawn(async move {
            futures::pin_mut!(stream);
            let sampled: Vec<_> = stream.take(messages.len()).collect().await;

            dbg!(&sampled);

            let sampled_client1_messages: Vec<_> = sampled
                .iter()
                .filter_map(|x| Some(x.ok().unwrap().id.topic_id == "client1"))
                .collect();

            let sampled_client2_messages: Vec<_> = sampled
                .iter()
                .filter_map(|x| Some(x.ok().unwrap().id.topic_id == "client2"))
                .collect();

            assert_eq!(
                sampled_client1_messages,
                ordered_client1_messages.clone(),
                "client1 messages not received in order"
            );
            assert_eq!(
                sampled_client2_messages,
                ordered_client2_messages.clone(),
                "client2 messages not received in order"
            );
        });

        let x = tokio::spawn(async move {
            let expected_rerequests = vec![
                ("client2".to_string(), 0),
                ("client2".to_string(), 1),
                ("client2".to_string(), 0),
                ("client1".to_string(), 1),
            ];
            let mut rerequests = vec![];

            while let Some(x) = r_rx.recv().await {
                rerequests.push(x);
            }

            assert_eq!(rerequests, expected_rerequests)
        });

        tokio::try_join!(x, y).unwrap();
    }
}
