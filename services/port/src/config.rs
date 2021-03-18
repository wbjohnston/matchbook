use std::env;

const EXCHANGE_ID_ENV_VAR_NAME: &str = "EXCHANGE_ID";

#[derive(Debug, Clone)]
pub struct Config {
    pub exchange_id: String,
}

pub fn source_config_from_env() -> Result<Config, Box<dyn std::error::Error>> {
    Ok(Config {
        exchange_id: env::var(EXCHANGE_ID_ENV_VAR_NAME)?,
    })
}
