# Observer

Solana network observability indexer (Rust + Postgres).

## Problem

Solana teams often lack a simple, self-hosted system that answers operational questions in near real time:

- Is failure rate spiking right now?
- Are blocks getting congested?
- Which programs are driving errors and compute pressure?
- Are we missing chain events after process restarts?
- Is a specific DeFi protocol showing abnormal fail rate or cost spikes?

Raw RPC calls and ad-hoc scripts do not provide reliable state continuity, replay safety, or analytics-ready storage.

## Solution

Observer implements the standard indexer split:

- WebSocket (`slotSubscribe`) for low-latency wake-up signals
- RPC (`getBlock`) for canonical finalized block data
- Postgres upserts for idempotent persistence
- Cursor (`last_indexed_slot`) for crash-safe resume
- Transaction-to-program mapping (`tx_programs`) for protocol risk monitoring

This makes indexing deterministic and replay-safe.

## Architecture

1. WebSocket stream emits new slot updates.
2. Indexer fetches full block data for each slot via RPC.
3. Parser computes core metrics (tx count, errors, fees, CU).
4. Storage layer upserts blocks/transactions and advances cursor.
5. SQL queries/views power dashboards and alert thresholds.

## What It Solves


- **Crash-safe continuity:** indexer resumes from DB cursor after restarts.
- **Historical traceability:** stores slot summaries (`tx_count`, `err_count`) and tx rows (`signature`, `is_error`, `fee_lamports`, `compute_units`).
- **Reliability visibility:** supports failed-tx rate and expensive-tx analysis via SQL.
- **Reprocessing safety:** uses upserts to avoid duplicate-row corruption.
- **Protocol risk visibility:** supports per-program failure/cost analysis for target DeFi protocols.

## Current Status

Implemented:

- Dockerized Postgres + schema bootstrap
- Slot ingestion (WS trigger + RPC fetch)
- Cursor-based backfill window
- Block and transaction persistence
- Fee/CU capture per transaction
- Transaction-to-program mapping (`tx_programs`)
- Continuous worker loop (runs until stopped)

Planned:

- Program-level attribution and error taxonomy
- KPI views + alerting hooks

## Why This Is Useful

Observer is not just a chain data collector. It is an operations product:

- Gives infra/DeFi teams measurable network health signals
- Reduces blind spots during RPC instability or process restarts
- Enables SLA-style monitoring with auditable historical data
- Provides a credible base for dashboards, alerts, and incident analysis

## Quick Start

```bash
cd /Users/raghavsharma/Documents/observer
cp .env.example .env
docker compose up -d
RUN_INDEXER=1 cargo run
```

## Database UI (Adminer)

Start services:

```bash
docker compose up -d
```

Open:

```text
http://localhost:8080
```

Login values:

- System: `PostgreSQL`
- Server: `postgres`
- Username: `observer`
- Password: `observer`
- Database: `observer`

## Use As A DeFi Risk Monitor

1. Configure target protocol program IDs in `.env`:

```bash
TARGET_PROGRAM_IDS=JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4,whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc
```

2. Start the continuous indexer:

```bash
RUN_INDEXER=1 cargo run
```

3. Stop with `Ctrl+C` when needed.

4. Query protocol risk metrics.

## Useful Queries

Recent slot integrity check:

```bash
docker exec -i observer-postgres psql -U observer -d observer -c "select b.slot, b.tx_count, coalesce(t.tx_rows, 0) as tx_rows from blocks b left join (select slot, count(*) as tx_rows from transactions group by slot) t on t.slot = b.slot order by b.slot desc limit 20;"
```

Recent tx cost/error sample:

```bash
docker exec -i observer-postgres psql -U observer -d observer -c "select signature, slot, is_error, fee_lamports, compute_units from transactions order by slot desc limit 20;"
```

Per-program risk summary (last 1000 slots):

```bash
docker exec -i observer-postgres psql -U observer -d observer -c "select tp.program_id, count(*) as txs, sum(case when t.is_error then 1 else 0 end) as failed_txs, round(100.0 * sum(case when t.is_error then 1 else 0 end)::numeric / nullif(count(*), 0), 2) as fail_rate_pct, round(avg(t.fee_lamports)::numeric, 2) as avg_fee_lamports, round(avg(t.compute_units)::numeric, 2) as avg_compute_units from tx_programs tp join transactions t on t.signature = tp.signature where tp.slot >= (select coalesce(max(slot), 0) - 1000 from blocks) group by tp.program_id order by fail_rate_pct desc, txs desc limit 20;"
```

Targeted protocol check (replace `<PROGRAM_ID>`):

```bash
docker exec -i observer-postgres psql -U observer -d observer -c "select t.slot, t.signature, t.is_error, t.fee_lamports, t.compute_units from tx_programs tp join transactions t on t.signature = tp.signature where tp.program_id = '<PROGRAM_ID>' order by t.slot desc limit 50;"
```

Recent failed transactions for one protocol:

```bash
docker exec -i observer-postgres psql -U observer -d observer -c "select t.slot, t.signature, t.fee_lamports, t.compute_units from tx_programs tp join transactions t on t.signature = tp.signature where tp.program_id = '<PROGRAM_ID>' and t.is_error = true order by t.slot desc limit 50;"
```

## Operational Notes

- Empty `TARGET_PROGRAM_IDS` means index and map all programs.
- Non-empty `TARGET_PROGRAM_IDS` means only those programs are written into `tx_programs`.
- The indexer remains continuous and catches up using `observer_cursor.last_indexed_slot`.

## Environment Variables

- `DATABASE_URL`: Postgres connection string
- `SOLANA_HTTP_URL`: Solana RPC HTTP endpoint
- `SOLANA_WS_URL`: Solana WebSocket endpoint
- `COMMITMENT`: `processed` | `confirmed` | `finalized`
- `TARGET_PROGRAM_IDS`: comma-separated program IDs to track in `tx_programs` (empty = track all)

## Development Approach

This repository is intentionally built in microtasks:

- small, testable increments
- explicit reasoning for each architectural choice
- production constraints introduced early (cursor, idempotency, recovery)
