use async_stream::stream;
use futures::{SinkExt, Stream, StreamExt};
use matchbook_types::*;

pub fn message_sequencer_stream(
    mut stream: impl futures::Stream<Item = Message> + Unpin,
    mut rerequest_sink: impl futures::Sink<usize> + Unpin,
    buf_size: usize,
    // need to add param to re-request messages
) -> impl Stream<Item = Message> {
    stream! {
        let mut next_expected_id = 0;
        let mut cursor = 0;
        let mut buf: Vec<Option<Message>> = vec![None; buf_size];
        let mut len = buf.len();

        // TODO(will): need to buffer by topic id
        while let Some(message) = stream.next().await {
            let id = message.id.topic_sequence_n as usize;

            if id == next_expected_id {
                yield message;

                next_expected_id += 1;

                // emit any cached messages
                loop {
                    if let Some(msg) = buf[cursor].take() {
                        cursor = (cursor + 1) % len;
                        next_expected_id += 1;
                        yield msg;
                    } else {
                        break
                    }
                }
            }
            // received element out of order
            else if id > next_expected_id {
                let offset = id - next_expected_id - 1;

                // if we are going to have an entry that may overwrite an element in our buffer, we need to resize
                let should_resize_buffer = offset > (len - 1);
                if should_resize_buffer {
                    let mut new_buf = vec![None; buf.len() * 2];
                    for (dst, x) in buf.iter().cloned().cycle().skip(cursor).take(buf.len()).enumerate() {
                        new_buf[dst] = x;
                    }
                    buf = new_buf;
                    len = buf.len();
                    cursor = 0;
                }
                let write_idx = (offset + cursor) % buf.len();
                buf[write_idx] = Some(message.clone());

                // re-request everything that we're missing
                let mut offset = 1;
                let mut scan_cursor = cursor;

                // TODO(will): error handling
                let _ = rerequest_sink.send(next_expected_id).await;
                while scan_cursor != write_idx {
                    if buf[scan_cursor].is_none() {
                        let id_to_rerequest = next_expected_id + offset;
                        let _ = rerequest_sink.send(id_to_rerequest).await;
                    }

                    offset += 1;
                    scan_cursor = (scan_cursor + 1) % len;
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
        let (r_tx, r_rx) = futures::channel::mpsc::unbounded();

        let stream = message_sequencer_stream(rx, r_tx, 2);

        let messages: Vec<_> = (0..5)
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

        let sending_order = vec![0, 3, 2, 4, 1, 2];

        for i in sending_order {
            tx.send(messages[i].clone()).await.unwrap();
        }

        drop(tx); // drop tx to close the channel

        let y = tokio::spawn(async move {
            futures::pin_mut!(stream);
            let sampled: Vec<_> = stream.take(messages.len()).collect().await;

            assert_eq!(messages, sampled);
        });

        let x = tokio::spawn(async move {
            let rerequests: Vec<_> = r_rx.collect().await;
            assert_eq!(rerequests, vec![1, 2, 1, 1])
        });

        tokio::try_join!(x, y).unwrap();
    }
}
