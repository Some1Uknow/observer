use tokio_postgres::Client;

pub async fn ensure_schema(client: &Client) -> Result<(), tokio_postgres::Error> {
    client
        .batch_execute(
            r#"
CREATE TABLE IF NOT EXISTS observer_cursor (
  id               SMALLINT PRIMARY KEY DEFAULT 1,
  last_indexed_slot BIGINT NOT NULL
);

INSERT INTO observer_cursor (id, last_indexed_slot)
VALUES (1, 0)
ON CONFLICT (id) DO NOTHING;

CREATE TABLE IF NOT EXISTS blocks (
  slot        BIGINT PRIMARY KEY,
  parent_slot BIGINT,
  blockhash   TEXT,
  block_time  BIGINT,
  tx_count    INT NOT NULL,
  err_count   INT NOT NULL
);

CREATE TABLE IF NOT EXISTS transactions (
  signature            TEXT PRIMARY KEY,
  slot                 BIGINT NOT NULL,
  is_error             BOOLEAN NOT NULL,
  fee_lamports         BIGINT,
  compute_units        BIGINT,
  first_error          TEXT
);

CREATE INDEX IF NOT EXISTS transactions_slot_idx ON transactions (slot);
"#,
        )
        .await
}

pub async fn get_last_indexed_slot(client: &Client) -> Result<i64, tokio_postgres::Error> {
    let row = client
        .query_one("SELECT last_indexed_slot FROM observer_cursor WHERE id = 1", &[])
        .await?;
    Ok(row.get::<_, i64>(0))
}

pub async fn set_last_indexed_slot(
    client: &Client,
    last_indexed_slot: i64,
) -> Result<(), tokio_postgres::Error> {
    client
        .execute(
            "UPDATE observer_cursor SET last_indexed_slot = $1 WHERE id = 1",
            &[&last_indexed_slot],
        )
        .await?;
    Ok(())
}

pub async fn upsert_block_memory(
    client: &Client,
    slot: i64,
    tx_count: i32,
    err_count: i32,
) -> Result<(), tokio_postgres::Error> {
    client.execute(
        r#"
        INSERT INTO blocks (slot, tx_count, err_count)
        VALUES ($1, $2, $3)
        ON CONFLICT (slot) DO UPDATE
        SET tx_count = EXCLUDED.tx_count,
        err_count = EXCLUDED.err_count
        "#, &[&slot, &tx_count, &err_count],).await?;

        Ok(())
}

pub async fn upsert_transaction_min(
    client: &Client,
    signature: &str,
    slot: i64,
    is_error: bool,
    fee_lamports: Option<i64>,
    compute_units: Option<i64>
) -> Result<(), tokio_postgres::Error> {
    client.execute(
        r#"
        INSERT INTO transactions (signature, slot, is_error, fee_lamports, compute_units)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (signature) DO UPDATE
        SET slot = EXCLUDED.slot,
        is_error = EXCLUDED.is_error,
        fee_lamports = EXCLUDED.fee_lamports,
        compute_units = EXCLUDED.compute_units
        "#, &[&signature, &slot, &is_error, &fee_lamports, &compute_units]).await?;
        Ok(())
}
