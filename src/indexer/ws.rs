use anyhow::Context;
use futures_util::StreamExt;
use solana_client::nonblocking::pubsub_client::PubsubClient;

pub async fn read_one_slot_event(ws_url: &str) -> anyhow::Result<Option<i64>> {
      let pubsub = PubsubClient::new(ws_url)
          .await
          .context("pubsub client creation failed")?;

        let mut last_seen_slot: Option<i64> = None;
   
    {
      let (mut slot_stream, unsubscribe) = pubsub
          .slot_subscribe()
          .await
          .context("slot subscribe failed")?;

    for i in 0..5 {
      if let Some(update) = slot_stream.next().await {
          println!("WS slot event #{i}: slot={}", update.slot);
          last_seen_slot = Some(update.slot as i64);
      } else {
          println!("WS stream ended at : {}", i);
          break;
      }}

      unsubscribe().await;
    }

      pubsub.shutdown().await.context("pubsub shutdown failed")?;
      Ok(last_seen_slot)
  }