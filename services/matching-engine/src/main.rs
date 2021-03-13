use futures::sink::SinkExt;
use futures::stream::StreamExt;
use matchbook_types::*;
use matchbook_util::*;
use matching_engine::*;
use std::net::{SocketAddr, SocketAddrV4};
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;
use tracing::*;

const DEFAULT_MULTICAST_ADDRESS: [u8; 4] = [239, 255, 42, 98];
const DEFAULT_MULTICAST_PORT: u16 = 50692;
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let (mut sink, mut stream) = {
        let socket = bind_multicast(
            &SocketAddrV4::new(IP_ALL.into(), DEFAULT_MULTICAST_PORT.into()),
            &SocketAddrV4::new(DEFAULT_MULTICAST_ADDRESS.into(), DEFAULT_MULTICAST_PORT),
        )?;
        let socket = UdpSocket::from_std(socket)?;
        let socket = UdpFramed::new(socket, MatchbookCodec::new());
        socket.split()
    };

    let multi_addr = SocketAddr::new(DEFAULT_MULTICAST_ADDRESS.into(), DEFAULT_MULTICAST_PORT);

    let mut engine = MatchingEngine::default();

    engine.create_symbol(['A', 'D', 'B', 'E']);
    engine.create_symbol(['C', 'O', 'I', 'N']); // hire me plz
    debug!("loaded symbols");

    loop {
        match stream.next().await {
            Some(Ok((message, addr))) => {
                let span = debug_span!("matching", ?addr);
                let _enter = span.enter();
                match message.kind {
                    MessageKind::LimitOrderSubmitRequest {
                        quantity,
                        price,
                        symbol,
                        side,
                    } => {
                        info!(
                            ?message.publisher_id,
                            message.topic_id,
                            ?side,
                            quantity,
                            ?symbol,
                            price,
                            "received limit order open request"
                        );
                        let fills = match engine.submit_limit_order(side, &symbol, price, quantity)
                        {
                            Ok(fills) => fills,
                            Err(e) => {
                                warn!("failed to submit order {}", e);
                                continue;
                            }
                        };

                        fills.iter().for_each(|execution| {
                            info!(
                                "executed {} shares of {:?} at {}",
                                execution.quantity, symbol, execution.price
                            )
                        });

                        let acknowledge_message = Message {
                            kind: MessageKind::LimitOrderSubmitRequestAcknowledge {
                                quantity,
                                price,
                                symbol,
                                side,
                            },
                            ..message
                        };

                        sink.send((acknowledge_message, multi_addr)).await?;
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
