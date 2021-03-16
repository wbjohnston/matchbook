use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

pub type UtcTimeStamp = DateTime<Utc>;
pub type AccountId = u64;
pub type ParticipantId = String;
pub type Price = usize;
pub type Quantity = usize;
pub type SymbolOwned = [char; 4];
pub type SymbolRef<'a> = &'a SymbolOwned;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Side {
    Bid,
    Ask,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde_as(as = "DisplayFromStr")]
    pub publisher_id: ServiceId,
    pub sending_time: UtcTimeStamp,
    pub topic_id: ParticipantId,
    /// Sequence number
    pub seq_n: u64,
    pub kind: MessageKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
        let mut split = s.split(":");
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ServiceKind {
    Port,
    MatchingEngine,
}

impl ServiceKind {
    pub fn as_str<'a>(self) -> &'a str {
        match self {
            ServiceKind::Port => "port",
            ServiceKind::MatchingEngine => "matching-engine",
        }
    }
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
            g.choose(&choices).unwrap().clone()
        }
    }

    quickcheck! {
        fn can_parse_service_identifier_from_str(kind: ServiceKind, n: u16) -> bool {
            let s = format!("{}:{}", kind.as_str(), n);
            ServiceId::from_str(&s).is_ok()
        }

        fn cant_parse_unknown_service_identifier_from_str(n: u16) -> bool {
            let s = format!("unknown:{}", n);
            ServiceId::from_str(&s).is_err()
        }
    }
}
