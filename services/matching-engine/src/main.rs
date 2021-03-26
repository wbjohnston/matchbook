#![deny(clippy::all)]
mod config;

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use matchbook_types::*;
use matchbook_util::*;
use matching_engine::*;
use tracing::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let config = config::source_config_from_env()?;

    let (mut sink, mut stream) = make_matchbook_streams(config.multicast_addr)?;

    let mut engine = MatchingEngine::default();

    engine.create_symbol(['A', 'D', 'B', 'E']);
    engine.create_symbol(['C', 'O', 'I', 'N']);
    debug!("loaded symbols");

    loop {
        match stream.next().await {
            Some(Ok(message)) => {
                #[allow(clippy::single_match)]
                match message.kind {
                    MessageKind::LimitOrderSubmitRequest {
                        quantity,
                        price,
                        symbol,
                        side,
                    } => {
                        info!(
                            ?message.id,
                            ?side,
                            quantity,
                            ?symbol,
                            price,
                            "received limit order open request"
                        );
                        let (id, executions) = match engine.submit_limit_order(
                            message.id.topic_id.clone(),
                            side,
                            &symbol,
                            price,
                            quantity,
                        ) {
                            Ok(fills) => fills,
                            Err(e) => {
                                warn!("failed to submit order {}", e);
                                continue;
                            }
                        };

                        let acknowledge_message = Message {
                            kind: MessageKind::LimitOrderSubmitRequestAcknowledge {
                                id,
                                quantity,
                                price,
                                symbol,
                                side,
                            },
                            ..message
                        };

                        sink.send(acknowledge_message).await?;

                        for execution in executions {
                            info!(?execution, "reporting execution");
                            let bidder_message = Message {
                                id: MessageId {
                                    publisher_id: config.service_id,
                                    topic_id: execution.bidder_id,
                                    // TODO(will): the matching engine needs to keep track internally what the current
                                    //              sequence number for each topic is. There's currently no way to know
                                    //              what this sequence number should be
                                    topic_sequence_n: 0,
                                },
                                kind: MessageKind::Execution {
                                    id: execution.bidder_order_id,
                                    price: execution.price,
                                    quantity: execution.quantity,
                                    side: Side::Bid,
                                    symbol,
                                },
                            };
                            let asker_message = Message {
                                id: MessageId {
                                    publisher_id: config.service_id,
                                    topic_id: execution.asker_id,
                                    // TODO(will): the matching engine needs to keep track internally what the current
                                    //              sequence number for each topic is. There's currently no way to know
                                    //              what this sequence number should be
                                    topic_sequence_n: 0,
                                },
                                kind: MessageKind::Execution {
                                    id: execution.asker_order_id,
                                    price: execution.price,
                                    side: Side::Ask,
                                    quantity: execution.quantity,
                                    symbol,
                                },
                            };

                            sink.send(bidder_message).await.unwrap();
                            sink.send(asker_message).await.unwrap();
                        }
                    }
                    _ => {}
                }
            }
            Some(Err(e)) => warn!("{}", e),
            None => break,
        }
    }

    Ok(())
}
