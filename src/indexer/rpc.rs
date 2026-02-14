use crate::config::Config;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

fn commitment_from_str(value: &str) -> CommitmentConfig {
    match value {
        "processed" => CommitmentConfig::processed(),
        "confirmed" => CommitmentConfig::confirmed(),
        _ => CommitmentConfig::finalized(),
    }
}

pub async fn print_current_slot(cfg: &Config) -> anyhow::Result<()> {
    let commitment = commitment_from_str(cfg.commitment.as_str());

    let rpc = RpcClient::new_with_commitment(cfg.solana_http_url.clone(), commitment);
    let current_slot = rpc.get_slot().await?;
    println!("Current Slot : {}", current_slot);

    Ok(())
}

pub async fn print_slot_tx_count(cfg: &Config, slot: u64) -> anyhow::Result<()> {
    let commitment = commitment_from_str(cfg.commitment.as_str());

    let rpc = RpcClient::new_with_commitment(cfg.solana_http_url.clone(), commitment);
    let block = rpc.get_block(slot).await?;
    let tx_count = block.transactions.len();

    println!("Slot={slot} tx_count={tx_count}");
    Ok(())
}
