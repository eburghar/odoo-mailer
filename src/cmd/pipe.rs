use crate::{config::Config, errors::HttpError};
use anyhow::{Error, Result};
use std::io::{self, Read};
use ureq::post;

pub fn cmd(config: &Config) -> Result<Option<String>> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    // sync post request the encoded email coming from stdin
    let url = format!("https://{}/mail_delivery/pipe", &config.host);
    let resp = post(&url)
        .set("X-Mail-Token", &config.token)
        .set("Content-Type", "text/plain")
        .send_string(&buffer);

    if resp.ok() {
        let text = resp.into_string()?;
        Ok(Some(text))
    } else {
        let code = resp.status();
        let details = resp.into_string()?;
        Err(Error::new(HttpError::new(code, &details)))
    }
}
