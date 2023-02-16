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

use std::{collections::HashMap, error::Error};

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

#[tokio::main]
async fn main() -> Result<()> {
    let client = reqwest::Client::builder().gzip(true).build()?;

    let sources = &get_categories(&client).await?;
    let small_sources = sources.split_at(100).0;

    let results: Vec<_> = futures::stream::iter(sources)
        .then(|s| async { process_category(&client, s.to_string()).await.unwrap() })
        .collect()
        .await;

    let all_parts = results.iter().flatten().cloned().collect::<Vec<_>>();

    let mut db = PartsDb {
        all_parts,
        cache: HashMap::new(),
    };

    //let redis_client = redis::Client::open("redis://127.0.0.1:6379/")?;
    //let mut redis_con = redis_client.get_connection()?;

    // configure_redis(&mut redis_con);

    // all_parts
    //     .iter()
    //     .progress()
    //     .for_each(|p| load_into_redis(&mut redis_con, p));

    // println!("{}", search_redis(&mut redis_con, "0603".to_string()).len());

    //println!("About to start server...");
    warm_cache(&mut db);
    webserver(db).await;

    Ok(())
}
