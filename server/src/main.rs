#![feature(async_closure)]

use bytes::{Buf, BufMut, Bytes, BytesMut};

use std::{
    error::Error,
    io::Read,
    path::{Path, PathBuf},
};

use flate2::read::GzDecoder;
use futures::future::join_all;
use reqwest::{Client, Url};
use serde_json::Value;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

struct Part {
    lcsc_id: String,
    manufacturer_id: String,
    price: f64,
    image_url: String,
    basic_or_extended: JLPCBStatus,
    stock: u64,
}

enum JLPCBStatus {
    Basic,
    Extended,
    Neither,
}

async fn my_get(c: &Client, url: String) -> Result<String, Box<dyn Error>> {
    let parsed_url = Url::parse(&url).expect("malformed url");
    let cache_path_str = vec!["cache", &parsed_url.path().replace("/", "-")].join("/");
    let cache_path = PathBuf::from(cache_path_str);
    println!("{:?}", cache_path);
    match File::open(cache_path.clone()).await {
        Err(_) => {
            let body = c.get(parsed_url).send().await?;
            let mut write_file = File::create(cache_path).await?;

            if url.contains(".json.gz") {
                let bytes = body.bytes().await?;
                let mut gz = GzDecoder::new(bytes.reader());
                let mut text: String = String::from("");
                gz.read_to_string(&mut text)?;
                write_file.write_all(text.as_bytes()).await?;
                return Ok(text);
            } else {
                let text = body.text().await?;
                write_file.write_all(text.as_bytes()).await?;
                return Ok(text);
            }
        }
        Ok(mut file) => {
            let mut contents: String = String::from("");
            file.read_to_string(&mut contents).await?;
            return Ok(contents);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::builder().gzip(true).build()?;

    let sources = get_categories(&client).await?;
    let small_sources = sources.split_at(3).0;

    // https://yaqwsx.github.io/jlcparts/data/Driver_ICsLED_Drivers.stock.json
    let _results = join_all(
        small_sources
            .iter()
            .map(|s| async { process_category(&client, s.to_string()).await }),
    )
    .await;

    Ok(())
}

async fn process_category(c: &Client, s: String) -> Result<(), Box<dyn Error>> {
    println!("processing {}", s);
    my_get(
        &c,
        format!("https://yaqwsx.github.io/jlcparts/data/{}.stock.json", s),
    )
    .await?;
    my_get(
        &c,
        format!("https://yaqwsx.github.io/jlcparts/data/{}.json.gz", s),
    )
    .await?;
    Ok(())
}

/*
    {
"categories": {
    "ADC/DAC/Data Conversion": {
    "ADC/DAC - Specialized": {
        "datahash": "681014911c466e50eb7619b884ad71b1af31f9eee3eea085cc160c36435d3206",
        "sourcename": "ADCakaDACakaData_ConversionADCakaDAC___Specialized",
        "stockhash": "673d7ad2b4ed80cc31394981595d44b3e56390099829daa877b5e7652eafe028"
    },
}
*/

async fn get_categories(c: &Client) -> Result<Vec<String>, Box<dyn Error>> {
    let out = my_get(
        c,
        String::from("https://yaqwsx.github.io/jlcparts/data/index.json"),
    )
    .await?;

    let v: Value = serde_json::from_str(&out)?;
    let sources: Vec<String> = match &v["categories"] {
        Value::Object(category) => category
            .values()
            .map(|subcat| match subcat {
                Value::Object(subcat) => subcat
                    .values()
                    .map(|r| {
                        r["sourcename"]
                            .as_str()
                            .expect("A sourcename was not a string?")
                    })
                    .collect::<Vec<_>>(),
                _ => todo!(),
            })
            .flatten()
            .map(|s| s.into())
            .collect(),
        _ => todo!(),
    };

    Ok(sources)
}
