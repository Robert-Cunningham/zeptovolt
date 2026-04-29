mod load_parts;
mod search;
mod server;
mod utils;

use load_parts::download_db;

use crate::server::webserver;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let db = download_db().await?;
    webserver(db).await;

    Ok(())
}
