use anyhow::Result;
use server::{download_db, server::webserver};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let db = download_db().await?;
    webserver(db).await;

    Ok(())
}
