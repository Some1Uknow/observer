pub mod rpc;
pub mod ws;

use crate::{config::Config, schema};
use std::collections::HashSet;
use tokio::time::{sleep, Duration};
use tokio_postgres::Client;

pub async fn run_slot_indexer(cfg: &Config, db: &Client) -> anyhow::Result<()> {
    println!("Starting continuous indexer loop...");
    let target_program_ids: HashSet<String> = cfg.target_program_ids.iter().cloned().collect();
    if target_program_ids.is_empty() {
        println!("Risk monitor mode: tracking all programs");
    } else {
        println!(
            "Risk monitor mode: tracking {} target program(s)",
            target_program_ids.len()
        );
    }

    loop {
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
            sleep(Duration::from_secs(1)).await;
            continue;
        }

        let max_slots_per_run = 20_i64;
        let end_slot = std::cmp::min(cursor + max_slots_per_run, head);

        for slot in (cursor + 1)..=end_slot {
            if let Some((tx_count, err_count, tx_summaries)) =
                rpc::print_slot_tx_count(cfg, slot as u64).await?
            {
                schema::upsert_block_memory(db, slot, tx_count, err_count).await?;
                for tx in &tx_summaries {
                    schema::upsert_transaction_min(
                        db,
                        &tx.signature,
                        slot,
                        tx.is_error,
                        tx.fee_lamports,
                        tx.compute_units,
                    )
                    .await?;
                    for program_id in &tx.program_ids {
                        if target_program_ids.is_empty() || target_program_ids.contains(program_id)
                        {
                            schema::upsert_tx_program(db, &tx.signature, slot, program_id).await?;
                        }
                    }
                }
                println!("Indexed #{slot} : {} tx rows", tx_summaries.len());
            } else {
                println!("Slot #{slot} unavailable/skipped");
            }

            schema::set_last_indexed_slot(db, slot).await?;
            println!("Cursor Updated to #{slot}");
        }

        sleep(Duration::from_millis(200)).await;
    }
}
