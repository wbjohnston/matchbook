#![deny(clippy::all)]
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

pub type UtcTimeStamp = DateTime<Utc>;
pub type AccountId = u64;
pub type ParticipantId = String;
pub type Price = usize;
pub type Quantity = usize;
pub type SymbolOwned = [char; 4];
pub type SymbolRef<'a> = &'a SymbolOwned;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    #[serde_as(as = "DisplayFromStr")]
    pub id: MessageId,
    pub kind: MessageKind,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageId {
    pub publisher_id: ServiceId,
    pub topic_id: String,
    pub topic_sequence_n: u64,
}

impl std::str::FromStr for MessageId {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut groups = s.split('.');

        let (publisher_id, topic_id, topic_sequence_n) = groups
            .next_tuple()
            .ok_or("insufficient number of elements in message id")?;

        let publisher_id = ServiceId::from_str(publisher_id)?;
        let topic_id = topic_id.to_string();
        let topic_sequence_n = topic_sequence_n.parse()?;

        Ok(MessageId {
            publisher_id,
            topic_id,
            topic_sequence_n,
        })
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            self.publisher_id, self.topic_id, self.topic_sequence_n
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageKind {
    LimitOrderSubmitRequest {
        side: Side,
        price: Price,
        quantity: Quantity,
        symbol: SymbolOwned,
    },
    LimitOrderSubmitRequestAcknowledge {
        side: Side,
        price: Price,
        quantity: Quantity,
        symbol: SymbolOwned,
    },
    // uses the id field of the message wrapper to determine what message to resend
    RetransmitRequest,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct ServiceId {
    pub kind: ServiceKind,
    pub number: u16,
}

impl std::fmt::Display for ServiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.kind, self.number)
    }
}

impl std::str::FromStr for ServiceId {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let mut split = s.split(':');
        let kind = if let Some(kind) = split.next() {
            ServiceKind::from_str(kind)?
        } else {
            return Err(format!("incorrectly formatted ServiceId '{}'", s).into());
        };

        let number = if let Some(num) = split.next() {
            num.parse()?
        } else {
            return Err(format!("incorrectly formatted ServiceId '{}'", s).into());
        };

        Ok(ServiceId { kind, number })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum ServiceKind {
    Port,
    MatchingEngine,
}

impl std::str::FromStr for ServiceKind {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        match s {
            "port" => Ok(ServiceKind::Port),
            "matching-engine" => Ok(ServiceKind::MatchingEngine),
            unknown => Err(format!("service kind '{}' is unknown", unknown).into()),
        }
    }
}

impl std::fmt::Display for ServiceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ServiceKind::*;
        let s = match self {
            Port => "port",
            MatchingEngine => "matching-engine",
        };

        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::quickcheck;
    use std::str::FromStr;

    impl quickcheck::Arbitrary for ServiceKind {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let choices = [ServiceKind::Port, ServiceKind::MatchingEngine];
            *g.choose(&choices).unwrap()
        }
    }

    quickcheck! {
        fn can_parse_service_identifier_from_str(kind: ServiceKind, n: u16) -> bool {
            let s = format!("{}:{}", kind, n);
            ServiceId::from_str(&s).is_ok()
        }

        fn cant_parse_unknown_service_identifier_from_str(n: u16) -> bool {
            let s = format!("unknown:{}", n);
            ServiceId::from_str(&s).is_err()
        }
    }
}
