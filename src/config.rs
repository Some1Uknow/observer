use anyhow::Context;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub solana_http_url: String,
    pub solana_ws_url: String,
    pub commitment: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;

        // Keep as Strings for now; we’ll map to Solana CommitmentConfig when we wire the client.
        let solana_http_url =
            env::var("SOLANA_HTTP_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".into());
        let solana_ws_url =
            env::var("SOLANA_WS_URL").unwrap_or_else(|_| "wss://api.devnet.solana.com/".into());
        let commitment = env::var("COMMITMENT").unwrap_or_else(|_| "finalized".into());

        Ok(Self {
            database_url,
            solana_http_url,
            solana_ws_url,
            commitment,
        })
    }
}
