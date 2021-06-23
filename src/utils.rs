use crate::{config::Config, errors::HttpError};
use anyhow::{Context, Error, Result};
use std::{
    env,
    fs::File,
    io::Write,
    os::unix::io::FromRawFd,
    path::{Path, PathBuf},
    process::Command,
};
use ureq::get;

pub fn which<P>(name: P) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(&name);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}

pub fn s6_ready(fd: Option<i32>) {
    if let Some(fd) = fd {
        let mut f = unsafe { File::from_raw_fd(fd) };
        let _ = write!(&mut f, "\n");
    }
}

pub enum MapType {
    Aliases,
    Transport,
}

impl MapType {
    pub fn write(&self, config: &Config, buf: &[u8]) -> Result<()> {
        let map = match self {
            MapType::Aliases => &config.aliases,
            MapType::Transport => &config.transport,
        };
        let mut file = File::create(map).with_context(|| format!("Can't open {}", map))?;
        // write the map file
        file.write_all(buf)?;
        // execute postmap
        if let Some(postmap) = which("postmap") {
            Command::new(postmap).args(&[map]).status()?;
        }
        Ok(())
    }

    pub fn get(&self, config: &Config) -> Result<String> {
        let path = match self {
            MapType::Aliases => "aliases",
            MapType::Transport => "transport",
        };
        let url = format!("https://{}/mail_delivery/{}", &config.host, path);
        let resp = get(&url).set("X-Mail-Token", &config.token).call();
        if resp.ok() {
            let text = resp.into_string()?;
            Ok(text)
        } else {
            let code = resp.status();
            let details = resp.into_string()?;
            Err(Error::new(HttpError::new(code, &details)))
        }
    }
}
