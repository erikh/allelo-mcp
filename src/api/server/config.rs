use serde::Deserialize;
use std::{net::SocketAddr, path::PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub listen: SocketAddr,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            listen: "127.0.0.1:8999".parse().unwrap(),
        }
    }
}

impl Config {
    pub fn from_file(filename: PathBuf) -> anyhow::Result<Self> {
        let r = std::fs::OpenOptions::new().read(true).open(filename)?;
        Ok(serde_yaml_ng::from_reader(r)?)
    }
}
