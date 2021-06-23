use crate::config::Config;
use crate::utils::MapType;
use anyhow::Result;

pub fn cmd(config: &Config) -> Result<Option<String>> {
    if let Ok(data) = MapType::Aliases.get(config) {
        MapType::Aliases.write(config, &data.as_bytes())?;
    }
    Ok(None)
}
