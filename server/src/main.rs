#![feature(async_closure)]

mod load_parts;
mod memorysearch;
mod server;
mod textsearch;
mod utils;

use futures::StreamExt;
use indicatif::ProgressIterator;
use memorysearch::{warm_cache, PartsDb};
use par_stream::ParStreamExt;
use regex::Regex;
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, error::Error, time::Instant};

use crate::{
    load_parts::{get_categories, process_category},
    server::webserver,
    textsearch::{configure_redis, load_into_redis, search_redis},
};

use anyhow::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Part {
    lcsc_id: String,
    manufacturer_id: String,
    description: String,
    price: f32,
    image_url: Option<String>,
    datasheet_url: String,
    basic_or_extended: String, // todo
    stock: u32,
}

enum JLPCBStatus {
    Basic,
    Extended,
    Neither,
}

async fn download_db() -> Result<PartsDb, anyhow::Error> {
    let sources = &get_categories().await?;
    // let small_sources = sources.split_at(100).0;

    println!("Loading parts...");
    let results: Vec<_> = futures::stream::iter(
        sources
            .into_iter()
            .map(|s| tokio::spawn(process_category(s.to_string()))),
    )
    .buffer_unordered(12)
    .map(|r| r.unwrap())
    .collect()
    .await;

    println!("Done.");

    let all_parts = results
        .iter()
        .flatten()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();

    let db = PartsDb {
        all_parts,
        cache: HashMap::new(),
        last_update: Instant::now(),
    };

    return Ok(db);
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = download_db().await?;
    webserver(db).await;

    Ok(())
}
