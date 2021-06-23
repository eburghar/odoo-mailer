use crate::{config::Config, utils::MapType};
use anyhow::Result;

pub fn cmd(config: &Config) -> Result<Option<String>> {
    if let Ok(data) = MapType::Transport.get(config) {
        let data = format!("{} lmtp:unix:{}", data, &config.socket);
        MapType::Transport.write(config, &data.as_bytes())?;
    }
    Ok(None)
}
