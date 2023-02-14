#![feature(async_closure)]

mod load_parts;
mod server;
mod textsearch;
mod utils;

use serde::{Deserialize, Serialize};

use std::error::Error;

use crate::{
    load_parts::get_categories,
    server::webserver,
    textsearch::{configure_redis, search_redis},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Part {
    lcsc_id: String,
    manufacturer_id: String,
    price: f64,
    image_url: Option<String>,
    basic_or_extended: String, // todo
    stock: u64,
}

enum JLPCBStatus {
    Basic,
    Extended,
    Neither,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::builder().gzip(true).build()?;

    let sources = &get_categories(&client).await?;
    let small_sources = sources.split_at(3).0;

    // let results: Vec<_> = futures::stream::iter(sources)
    //     .then(|s| async { process_category(&client, s.to_string()).await.unwrap() })
    //     .collect()
    //     .await;

    // let all_parts = results.iter().flatten().collect::<Vec<_>>();

    let redis_client = redis::Client::open("redis://127.0.0.1:6379/")?;
    let mut redis_con = redis_client.get_connection()?;

    configure_redis(&mut redis_con);

    // all_parts
    //     .iter()
    //     .progress()
    //     .for_each(|p| load_into_redis(&mut redis_con, p));

    // println!("{}", search_redis(&mut redis_con, "0603".to_string()).len());

    webserver(redis_con).await;

    Ok(())
}
