pub mod rpc;
pub mod ws;

use crate::{config::Config, schema};
use tokio_postgres::Client;

pub async fn run_slot_indexer(cfg: &Config, db: &Client) -> anyhow::Result<()> {
    rpc::print_current_slot(cfg).await?;

    match ws::read_one_slot_event(&cfg.solana_ws_url).await {
        Ok(_) => {}
        Err(err) => println!("WS unavailable, continuing with RPC polling path: {err}"),
    } 

     let cursor = schema::get_last_indexed_slot(db).await?;
      let head = rpc::get_current_slot(cfg).await? as i64;
      println!("cursor={cursor} head={head}");

      if cursor >= head {
          println!("No finalized slots pending");
          return Ok(());
      }

     let max_slots_per_run = 20_i64;
     let end_slot = std::cmp::min(cursor + max_slots_per_run, head);

     for slot in (cursor + 1)..=end_slot {
        if rpc::print_slot_tx_count(cfg, slot as u64).await? {
            println!("Indexed #{slot}")
        } else {
            println!("Slot #{slot} unavailable/skipped");
        }

        schema::set_last_indexed_slot(db, slot).await?;
        println!("Cursor Updated to #{slot}");
     }

      Ok(())
  }


