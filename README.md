# Observer

Solana network observability indexer (Rust + Postgres).

## Problem

Solana teams often lack a simple, self-hosted system that answers operational questions in near real time:

- Is failure rate spiking right now?
- Are blocks getting congested?
- Which programs are driving errors and compute pressure?
- Are we missing chain events after process restarts?

Raw RPC calls and ad-hoc scripts do not provide reliable state continuity, replay safety, or analytics-ready storage.

## Solution

Observer implements the standard indexer split:

- WebSocket (`slotSubscribe`) for low-latency wake-up signals
- RPC (`getBlock`) for canonical finalized block data
- Postgres upserts for idempotent persistence
- Cursor (`last_indexed_slot`) for crash-safe resume

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

## Current Status

Implemented:

- Dockerized Postgres + schema bootstrap
- Slot ingestion (WS trigger + RPC fetch)
- Cursor-based backfill window
- Block and transaction persistence
- Fee/CU capture per transaction
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

## Useful Queries

Recent slot integrity check:

```bash
docker exec -i observer-postgres psql -U observer -d observer -c "select b.slot, b.tx_count, coalesce(t.tx_rows, 0) as tx_rows from blocks b left join (select slot, count(*) as tx_rows from transactions group by slot) t on t.slot = b.slot order by b.slot desc limit 20;"
```

Recent tx cost/error sample:

```bash
docker exec -i observer-postgres psql -U observer -d observer -c "select signature, slot, is_error, fee_lamports, compute_units from transactions order by slot desc limit 20;"
```

## Environment Variables

- `DATABASE_URL`: Postgres connection string
- `SOLANA_HTTP_URL`: Solana RPC HTTP endpoint
- `SOLANA_WS_URL`: Solana WebSocket endpoint
- `COMMITMENT`: `processed` | `confirmed` | `finalized`

## Development Approach

This repository is intentionally built in microtasks:

- small, testable increments
- explicit reasoning for each architectural choice
- production constraints introduced early (cursor, idempotency, recovery)
