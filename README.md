# Observer

Real-time Solana network observability pipeline built with Rust and Postgres.

## Problem

Solana teams often lack a simple, self-hosted system that answers operational questions in near real time:

- Is failure rate spiking right now?
- Are blocks getting congested?
- Which programs are driving errors and compute pressure?
- Are we missing chain events after process restarts?

Raw RPC calls and ad-hoc scripts do not provide reliable state continuity, replay safety, or analytics-ready storage.

## Solution

Observer is a production-style slot indexer:

- Uses WebSocket (`slotSubscribe`) for low-latency slot signals
- Uses RPC (`getBlock`) for canonical block + transaction data
- Stores normalized records in Postgres for analytics and alerting
- Persists an indexing cursor (`last_indexed_slot`) for crash-safe resume

This design separates real-time event detection from authoritative data retrieval, which is the standard architecture used by production indexers.

## Architecture

1. WebSocket stream emits new slot updates.
2. Indexer fetches full block data for each slot via RPC.
3. Parser computes core metrics (tx count, errors, fees, CU).
4. Storage layer upserts blocks/transactions and advances cursor.
5. SQL queries/views power dashboards and alert thresholds.

## Current Status

Implemented:

- Environment + dependency setup
- Dockerized Postgres
- Schema bootstrap (`blocks`, `transactions`, `observer_cursor`)
- RPC connectivity with configurable commitment
- WebSocket slot subscription (event stream)
- Cursor write path (`last_indexed_slot` updates)

In progress:

- Per-slot `getBlock` ingestion and DB inserts
- Startup backfill from cursor to latest slot
- Idempotent retry/recovery behavior

Planned:

- Aggregated metrics tables/views (TPS, failure rate, fee/CU trends)
- Alert hooks for congestion and error spikes
- Program-level attribution and hot-path analysis

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
