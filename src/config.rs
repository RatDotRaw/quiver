use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub broadcaster_id: String,
    pub events: Vec<String>,
    pub rabbit_mq: RabbitMQ,
    #[serde(rename = "AppKeys")]
    pub app_keys: AppKeys,
}

#[derive(Deserialize)]
pub struct RabbitMQ {
    pub rabbit_mq_url: String,
    pub exchange_name: String,
    pub routing_key: String,
}

#[derive(Deserialize)]
pub struct AppKeys {
    pub client_id: String,
    pub client_secret: String,
}

pub fn deserialize() -> Config {
    let config_content = fs::read_to_string("config.toml").expect("'config.toml' file not found");
    let config: Config =
        toml::from_str(&config_content).expect("failed to deserialize config file");
    config
}
