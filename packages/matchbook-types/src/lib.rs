use serde::{Deserialize, Serialize};

pub type Price = usize;
pub type Quantity = usize;
pub type SymbolOwned = String;
pub type SymbolRef<'a> = &'a str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    LimitOrderSubmitRequest {
        side: Side,
        price: Price,
        quantity: Quantity,
        symbol: SymbolOwned,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ServiceIdentifier {
    kind: ServiceKind,
    number: u16,
}

impl std::str::FromStr for ServiceIdentifier {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let mut split = s.split(":");
        let kind = if let Some(kind) = split.next() {
            ServiceKind::from_str(kind)?
        } else {
            return Err(format!("incorrectly formatted ServiceIdentifier '{}'", s).into());
        };

        let number = if let Some(num) = split.next() {
            num.parse()?
        } else {
            return Err(format!("incorrectly formatted ServiceIdentifier '{}'", s).into());
        };

        Ok(ServiceIdentifier { kind, number })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ServiceKind {
    Port,
}

impl ServiceKind {
    pub fn as_str<'a>(self) -> &'a str {
        match self {
            ServiceKind::Port => "port",
        }
    }
}

impl std::str::FromStr for ServiceKind {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        match s {
            "port" => Ok(ServiceKind::Port),
            unknown => Err(format!("service kind '{}' is unknown", unknown).into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::quickcheck;
    use std::str::FromStr;

    impl quickcheck::Arbitrary for ServiceKind {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let choices = [ServiceKind::Port];
            g.choose(&choices).unwrap().clone()
        }
    }

    quickcheck! {
        fn can_parse_service_identifier_from_str(kind: ServiceKind, n: u16) -> bool {
            let s = format!("{}:{}", kind.as_str(), n);
            ServiceIdentifier::from_str(&s).is_ok()
        }

        fn cant_parse_unknown_service_identifier_from_str(n: u16) -> bool {
            let s = format!("unknown:{}", n);
            ServiceIdentifier::from_str(&s).is_err()
        }
    }
}
