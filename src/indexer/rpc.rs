use crate::config::Config;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcBlockConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::{EncodedTransaction, EncodedTransactionWithStatusMeta};
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct TxSummary {
    pub signature: String,
    pub is_error: bool,
    pub fee_lamports: Option<i64>,
}

fn commitment_from_str(value: &str) -> CommitmentConfig {
    match value {
        "processed" => CommitmentConfig::processed(),
        "confirmed" => CommitmentConfig::confirmed(),
        _ => CommitmentConfig::finalized(),
    }
}

fn extract_signature(tx: &EncodedTransactionWithStatusMeta) -> Option<String> {
    match &tx.transaction {
        EncodedTransaction::Json(ui_tx) => ui_tx.signatures.first().cloned(),
        EncodedTransaction::Accounts(accounts) => accounts.signatures.first().cloned(),
        _ => None,
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

pub async fn print_slot_tx_count(
    cfg: &Config,
    slot: u64,
) -> anyhow::Result<Option<(i32, i32, Vec<TxSummary>)>> {
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
                let txs = block
                    .transactions
                    .as_ref()
                    .map_or(&[][..], |v| v.as_slice());
                let tx_count = txs.len() as i32;

                let err_count = txs
                    .iter()
                    .filter(|tx| tx.meta.as_ref().is_some_and(|m| m.err.is_some()))
                    .count() as i32;

                let mut tx_summaries: Vec<TxSummary> = Vec::new();

                for (idx, tx) in txs.iter().enumerate() {
                    let signature = extract_signature(tx)
                        .unwrap_or_else(|| format!("missing-signature-{slot}-{idx}"));
                    let is_error = tx.meta.as_ref().is_some_and(|m| m.err.is_some());

                    let fee_lamports = tx.meta.as_ref().and_then(|m| i64::try_from(m.fee).ok());
                    tx_summaries.push(TxSummary {
                        signature,
                        is_error,
                        fee_lamports,
                    });
                }

                println!("slot={slot} tx_count={tx_count} err_count={err_count}");
                return Ok(Some((tx_count, err_count, tx_summaries)));
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
    Ok(None)
}
