pub mod rpc;
pub mod ws;

use crate::{config::Config, schema};
use std::collections::HashSet;
use tokio::time::{sleep, Duration};
use tokio_postgres::Client;

const IDLE_SLEEP_SECS: u64 = 1;
const LOOP_SLEEP_MILLIS: u64 = 200;
const MAX_SLOTS_PER_ITERATION: i64 = 20;

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
        rpc::log_current_head_slot(cfg).await?;

        match ws::collect_slot_burst(&cfg.solana_ws_url).await {
            Ok(_) => {}
            Err(err) => println!("WS unavailable, continuing with RPC polling path: {err}"),
        }

        let cursor = schema::load_last_indexed_slot(db).await?;
        let head = rpc::fetch_current_slot(cfg).await? as i64;
        println!("cursor={cursor} head={head}");

        if cursor >= head {
            println!("No finalized slots pending");
            sleep(Duration::from_secs(IDLE_SLEEP_SECS)).await;
            continue;
        }

        let end_slot = std::cmp::min(cursor + MAX_SLOTS_PER_ITERATION, head);

        for slot in (cursor + 1)..=end_slot {
            if let Some(slot_metrics) = rpc::fetch_slot_metrics(cfg, slot as u64).await? {
                schema::upsert_block_summary(
                    db,
                    slot,
                    slot_metrics.tx_count,
                    slot_metrics.err_count,
                )
                .await?;
                for tx in &slot_metrics.tx_summaries {
                    schema::upsert_transaction_summary(
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
                println!(
                    "Indexed #{slot} : {} tx rows",
                    slot_metrics.tx_summaries.len()
                );
            } else {
                println!("Slot #{slot} unavailable/skipped");
            }

            schema::save_last_indexed_slot(db, slot).await?;
            println!("Cursor Updated to #{slot}");
        }

        sleep(Duration::from_millis(LOOP_SLEEP_MILLIS)).await;
    }
}
