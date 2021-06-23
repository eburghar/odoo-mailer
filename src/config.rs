use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs::OpenOptions;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub host: String,
    pub token: String,
    #[serde(default = "default_aliases")]
    pub aliases: String,
    #[serde(default = "default_transport")]
    pub transport: String,
    #[serde(default = "default_socket")]
    pub socket: String,
}

fn default_aliases() -> String {
    "/etc/postfix/virtual_alias_odoo".to_string()
}

fn default_transport() -> String {
    "/etc/postfix/transport_odoo".to_string()
}

fn default_socket() -> String {
    "/var/spool/postfix/private/odoo-lmtp".to_string()
}

pub fn get_config(config: &str) -> Result<Config> {
    // open configuration file
    let file = OpenOptions::new()
        .read(true)
        .open(&config)
        .with_context(|| format!("Can't open {}", &config))?;
    // deserialize configuration
    let config: Config =
        serde_yaml::from_reader(file).with_context(|| format!("Can't read {}", &config))?;
    Ok(config)
}
