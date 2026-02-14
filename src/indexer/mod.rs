pub mod rpc;
pub mod ws;

use crate::config::Config;
use tokio_postgres::Client;

pub async fn run_slot_indexer(cfg: &Config, _db: &Client) -> anyhow::Result<()> {
    rpc::print_current_slot(cfg).await?;
    ws::read_one_slot_event(&cfg.solana_ws_url).await?;
    Ok(())
}