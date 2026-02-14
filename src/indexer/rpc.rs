use crate::config::Config;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcBlockConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use tokio::time::sleep;

fn commitment_from_str(value: &str) -> CommitmentConfig {
    match value {
        "processed" => CommitmentConfig::processed(),
        "confirmed" => CommitmentConfig::confirmed(),
        _ => CommitmentConfig::finalized(),
    }
}

pub async fn get_current_slot(cfg: &Config) -> anyhow::Result<u64> {
    let commitment = commitment_from_str(cfg.commitment.as_str());
    let rpc = RpcClient::new_with_commitment(cfg.solana_http_url.clone(), commitment);
    Ok(rpc.get_slot().await?)
}

pub async fn print_current_slot(cfg: &Config) -> anyhow::Result<()> {
    let commitment = commitment_from_str(cfg.commitment.as_str());

    let rpc = RpcClient::new_with_commitment(cfg.solana_http_url.clone(), commitment);
    let current_slot = rpc.get_slot().await?;
    println!("Current Slot : {}", current_slot);

    Ok(())
}

pub async fn print_slot_tx_count(cfg: &Config, slot: u64) -> anyhow::Result<bool> {
    use tokio::time::Duration;

    let commitment = commitment_from_str(cfg.commitment.as_str());
    let rpc = RpcClient::new_with_commitment(cfg.solana_http_url.clone(), commitment);
    
    for attempt in 1..=5 {

        let block_config = RpcBlockConfig {
            max_supported_transaction_version: Some(0),
            ..RpcBlockConfig::default()
        };

        match rpc.get_block_with_config(slot, block_config).await {
            Ok(block) => {
                let tx_count = block.transactions.as_ref().map_or(0, |txs| txs.len());
                println!("slot={slot} tx_count={tx_count}");
                return Ok(true);
            }
            Err(err) => {
                let msg = err.to_string();
                if msg.contains("Block not available for slot") {
                    println!("slot={slot} not available yet (attempt {attempt}/10)");
                    sleep(Duration::from_millis(500)).await;
                    continue;
                }
                return Err(err.into());
            }
        }
    }

    println!("slot={slot} still unavailable after retries; keep cursor unchanged");
    Ok(false)
}
