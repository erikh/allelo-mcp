use allelo_mcp::api::server::{Config, Server};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // FIXME: replace this with clap later
    let config = if std::env::args().len() < 2 {
        Config::default()
    } else {
        Config::from_file(
            std::env::args()
                .skip(1)
                .next()
                .expect("Could not parse configuration file")
                .into(),
        )?
    };

    Server::new(config).await?.start().await
}
