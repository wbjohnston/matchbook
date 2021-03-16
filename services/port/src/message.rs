use fixer_upper::{Message as FixMessage, MessageType as FixMessageType};
use itertools::Itertools;
use matchbook_types::*;

pub fn fix_message_into_matchbook_message(
    msg: FixMessage,
    service_id: ServiceId,
) -> Result<Message, Box<dyn std::error::Error>> {
    use FixMessageType::*;
    use MessageKind::*;

    let symbol = msg.body.symbol.ok_or("missing required field 'Symbol'")?;

    Ok(match msg.header.msg_type {
        NewOrderSingle => Message {
            kind: LimitOrderSubmitRequest {
                price: msg.body.price.ok_or("missing required field 'Price'")? as Price,
                quantity: msg
                    .body
                    .order_qty
                    .ok_or("missing required field 'OrderQty'")?
                    as Quantity,
                side: Side::Bid,
                symbol: {
                    let (first, second, third, fourth) = symbol
                        .chars()
                        .next_tuple()
                        .ok_or("incorrect length for field 'Symbol'")?;
                    [first, second, third, fourth]
                },
            },
            id: MessageId {
                publisher_id: service_id,
                topic_id: msg.header.sender_comp_id,
                topic_sequence_n: msg.header.msg_seq_num,
            },
            sending_time: msg.header.sending_time,
        },
        x => unimplemented!("{:?}", x),
    })
}

pub fn matchbook_message_into_fix_message(msg: Message) -> FixMessage {
    match msg.kind {
        MessageKind::LimitOrderSubmitRequest {
            price,
            side,
            symbol,
            quantity,
            ..
        } => FixMessage {
            header: fixer_upper::Header {
                begin_string: fixer_upper::BeginString::Fix_4_4,
                body_length: None,
                msg_type: FixMessageType::NewOrderSingle,
                sender_comp_id: String::from("matchbook"),
                target_comp_id: msg.id.topic_id.to_string(),
                msg_seq_num: msg.id.topic_sequence_n,
                sending_time: msg.sending_time,
            },
            body: fixer_upper::Body {
                cl_ord_id: Some(msg.id.topic_id.to_string()),
                handl_inst: Some(fixer_upper::HandleInstruction::ManualOrderBestExecution),
                symbol: Some(symbol.iter().collect()),
                side: Some(match side {
                    Side::Ask => fixer_upper::Side::Sell,
                    Side::Bid => fixer_upper::Side::Buy,
                }),
                transact_time: Some(msg.sending_time),
                ord_type: Some(fixer_upper::OrderType::Limit),
                order_qty: Some(quantity as f64),
                price: Some(price as f64),
            },
            trailer: fixer_upper::Trailer {
                signature_length: None,
                signature: None,
            },
        },
        x => unimplemented!("{:?}", x),
    }
}
