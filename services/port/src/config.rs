use std::str::FromStr;
use std::{env, net::SocketAddr};

use matchbook_types::ServiceId;

const MULTICAST_ADDR_ENV_VAR_NAME: &str = "MULTICAST_ADDR";
const SERVICE_ID_ENV_VAR_NAME: &str = "SERVICE_ID";
const EXCHANGE_ID_ENV_VAR_NAME: &str = "EXCHANGE_ID";

#[derive(Debug, Clone)]
pub struct Config {
    pub service_id: ServiceId,
    pub multicast_addr: SocketAddr,
    pub exchange_id: String,
}

pub fn source_config_from_env() -> Result<Config, Box<dyn std::error::Error>> {
    Ok(Config {
        service_id: env::var(SERVICE_ID_ENV_VAR_NAME)
            .map(|x| ServiceId::from_str(x.as_str()))??,
        multicast_addr: env::var(MULTICAST_ADDR_ENV_VAR_NAME).map(|x| x.parse())??,
        exchange_id: env::var(EXCHANGE_ID_ENV_VAR_NAME)?,
    })
}
