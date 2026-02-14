# Observer (Solana Network Observability Indexer)

Goal: a production-style **slot indexer** that ingests blocks and writes to Postgres so you can build dashboards/alerts:
- tx count, failed-tx rate
- fees and compute usage trends
- (later) top programs by usage/errors

## Local dev

1) Start Postgres

```bash
cd /Users/raghavsharma/Documents/observer
docker compose up -d
```

2) Configure env

```bash
cp .env.example .env
```

3) Run

```bash
cargo run
```

## Roadmap (learning loop)
- Persist cursor (`last_indexed_slot`) for crash-safe resume
- WS slot stream + RPC `getBlock` fetch
- Backfill: on startup, catch up from cursor to current slot
- Dedupe + idempotent inserts
- Metrics views (SQL) for dashboards/alerts
