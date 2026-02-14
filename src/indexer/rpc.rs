use crate::config::Config;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

pub async fn print_current_slot(cfg: &Config) -> anyhow::Result<()> {
    let commitment = match cfg.commitment.as_str() {
        "processed" => CommitmentConfig::processed(),
        "confirmed" => CommitmentConfig::confirmed(),
        _ => CommitmentConfig::finalized()
    };

    let rpc = RpcClient::new_with_commitment(cfg.solana_http_url.clone(), commitment);
    let current_slot = rpc.get_slot().await?;
    println!("Current Slot : {}", current_slot);

    Ok(())
}