mod config;
mod indexer;
mod schema;

use anyhow::Context;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let cfg = config::Config::from_env().context("load config")?;

    let (client, connection) = tokio_postgres::connect(&cfg.database_url, NoTls)
        .await
        .context("connect to Postgres")?;

    tokio::spawn(async move {
        if let Err(err) = connection.await {
            eprintln!("postgres connection error: {err}");
        }
    });

    schema::init_schema(&client)
        .await
        .context("ensure schema")?;

    let cursor = schema::load_last_indexed_slot(&client).await?;
    println!("observer ready (last_indexed_slot={cursor})");
    println!(
        "solana config loaded (commitment={}, target_programs={})",
        cfg.commitment,
        cfg.target_program_ids.len()
    );

    if std::env::var("RUN_INDEXER").as_deref() == Ok("1") {
        indexer::run_slot_indexer(&cfg, &client)
            .await
            .context("run indexer")?;
    } else {
        println!("Set RUN_INDEXER=1 to start indexing.");
    }
    Ok(())
}
