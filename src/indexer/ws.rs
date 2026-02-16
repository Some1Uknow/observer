use anyhow::Context;
use futures_util::StreamExt;
use solana_client::nonblocking::pubsub_client::PubsubClient;

pub async fn collect_slot_burst(ws_url: &str) -> anyhow::Result<Option<i64>> {
    let pubsub = PubsubClient::new(ws_url)
        .await
        .context("pubsub client creation failed")?;

    let mut first_seen_slot: Option<i64> = None;
    let mut last_seen_slot: Option<i64> = None;
    let mut seen_count = 0usize;

    {
        let (mut slot_stream, unsubscribe) = pubsub
            .slot_subscribe()
            .await
            .context("slot subscribe failed")?;

        for _ in 0..5 {
            if let Some(update) = slot_stream.next().await {
                let slot = update.slot as i64;
                if first_seen_slot.is_none() {
                    first_seen_slot = Some(slot);
                }
                last_seen_slot = Some(slot);
                seen_count += 1;
            } else {
                break;
            }
        }

        unsubscribe().await;
    }

    match (first_seen_slot, last_seen_slot) {
        (Some(first), Some(last)) => {
            println!("WS burst: events={seen_count} first_slot={first} last_slot={last}");
        }
        _ => println!("WS burst: no slot events received"),
    }

    pubsub.shutdown().await.context("pubsub shutdown failed")?;
    Ok(last_seen_slot)
}
