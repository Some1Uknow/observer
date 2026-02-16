use crate::config::Config;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcBlockConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::{
    EncodedTransaction, EncodedTransactionWithStatusMeta, UiInstruction, UiMessage,
    UiParsedInstruction,
};
use std::collections::BTreeSet;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct TxSummary {
    pub signature: String,
    pub is_error: bool,
    pub fee_lamports: Option<i64>,
    pub compute_units: Option<i64>,
    pub program_ids: Vec<String>,
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

fn extract_program_ids(tx: &EncodedTransactionWithStatusMeta) -> Vec<String> {
    let mut program_ids = BTreeSet::new();
    if let EncodedTransaction::Json(ui_tx) = &tx.transaction {
        match &ui_tx.message {
            UiMessage::Raw(raw) => {
                for instruction in &raw.instructions {
                    if let Some(program_id) =
                        raw.account_keys.get(instruction.program_id_index as usize)
                    {
                        program_ids.insert(program_id.clone());
                    }
                }
            }
            UiMessage::Parsed(parsed) => {
                for instruction in &parsed.instructions {
                    match instruction {
                        UiInstruction::Compiled(compiled) => {
                            if let Some(account) =
                                parsed.account_keys.get(compiled.program_id_index as usize)
                            {
                                program_ids.insert(account.pubkey.clone());
                            }
                        }
                        UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(decoded)) => {
                            program_ids.insert(decoded.program_id.clone());
                        }
                        UiInstruction::Parsed(UiParsedInstruction::Parsed(parsed_ix)) => {
                            program_ids.insert(parsed_ix.program_id.clone());
                        }
                    }
                }
            }
        }
    }
    program_ids.into_iter().collect()
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

                    let compute_units = tx.meta.as_ref().and_then(|m| {
                        let maybe_cu: Option<u64> = m.compute_units_consumed.clone().into();
                        maybe_cu.and_then(|cu| i64::try_from(cu).ok())
                    });

                    tx_summaries.push(TxSummary {
                        signature,
                        is_error,
                        fee_lamports,
                        compute_units,
                        program_ids: extract_program_ids(tx),
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
