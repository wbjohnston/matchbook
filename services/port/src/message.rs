use fixer_upper::{Message as FixMessage, MessageType as FixMessageType};
use matchbook_types::*;

pub fn fix_message_into_matchbook_message(msg: FixMessage, service_id: ServiceId) -> Message {
    use FixMessageType::*;
    use MessageKind::*;

    // TODO(will): handle when this is null
    let symbol = msg.body.symbol.unwrap();

    match msg.header.msg_type {
        NewOrderSingle => Message {
            kind: LimitOrderSubmitRequest {
                price: msg.body.price.unwrap() as Price,
                quantity: msg.body.order_qty.unwrap() as Quantity,
                side: Side::Bid,
                symbol: [
                    symbol.chars().nth(0).unwrap(),
                    symbol.chars().nth(1).unwrap(),
                    symbol.chars().nth(2).unwrap(),
                    symbol.chars().nth(3).unwrap(),
                ],
            },
            publisher_id: service_id,
            sending_time: msg.header.sending_time,
            topic_id: msg.header.sender_comp_id,
            seq_n: msg.header.msg_seq_num,
        },
        x => unimplemented!("{:?}", x),
    }
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
                target_comp_id: format!("{}", msg.topic_id),
                msg_seq_num: msg.seq_n,
                sending_time: msg.sending_time,
            },
            body: fixer_upper::Body {
                cl_ord_id: Some(format!("{}", msg.topic_id)),
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
