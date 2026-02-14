pub mod rpc;
pub mod ws;

use crate::{config::Config, schema};
use tokio_postgres::Client;

pub async fn run_slot_indexer(cfg: &Config, db: &Client) -> anyhow::Result<()> {
    rpc::print_current_slot(cfg).await?;

    if let Some(last_slot) = ws::read_one_slot_event(&cfg.solana_ws_url).await? {
        schema::set_last_indexed_slot(db, last_slot).await?;
        println!("Cursor Updated to #{last_slot}");
        rpc::print_slot_tx_count(cfg, last_slot as u64).await?;
    } else {
        println!("No slot events received, cursor unchanged");
    }

    Ok(())
}
