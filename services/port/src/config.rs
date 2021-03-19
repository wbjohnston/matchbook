use std::str::FromStr;
use std::{env, net::SocketAddr};

use matchbook_types::ServiceId;

const MULTICAST_ADDR_ENV_VAR_NAME: &str = "MULTICAST_ADDR";
const SERVICE_ID_ENV_VAR_NAME: &str = "SERVICE_ID";
const EXCHANGE_ID_ENV_VAR_NAME: &str = "EXCHANGE_ID";
const TLS_CERT_ENV_VAR_NAME: &str = "TLS_CERT";
const TLS_CERT_KEY_ENV_VAR_NAME: &str = "TLS_CERT_KEY";

#[derive(Debug, Clone)]
pub struct Config {
    pub service_id: ServiceId,
    pub multicast_addr: SocketAddr,
    pub exchange_id: String,
    pub tls_cert: String,
    pub tls_cert_key: String,
}

pub fn source_config_from_env() -> Result<Config, Box<dyn std::error::Error>> {
    Ok(Config {
        service_id: env::var(SERVICE_ID_ENV_VAR_NAME)
            .map(|x| ServiceId::from_str(x.as_str()))??,
        multicast_addr: env::var(MULTICAST_ADDR_ENV_VAR_NAME).map(|x| x.parse())??,
        exchange_id: env::var(EXCHANGE_ID_ENV_VAR_NAME)?,
        tls_cert: env::var(TLS_CERT_ENV_VAR_NAME)?,
        tls_cert_key: env::var(TLS_CERT_KEY_ENV_VAR_NAME)?,
    })
}
