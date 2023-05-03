use std::{
    fs,
    path::PathBuf,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use bytes::Buf;
use flate2::read::GzDecoder;
use futures::StreamExt;
use reqwest::{Client, Url};
use serde_json::Value;
use std::{error::Error, io::Read};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

use anyhow::Result;

pub async fn cached_get(url: String) -> Result<String> {
    let day = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() / 86400;
    let day_str = format!("{}", day);

    let parsed_url = Url::parse(&url).expect("malformed url");
    let cache_path_str = vec!["cache", &day_str, &parsed_url.path().replace("/", "-")].join("/");
    let cache_path = PathBuf::from(cache_path_str);
    fs::create_dir_all(cache_path.parent().unwrap())?;

    // println!("{:?}", cache_path);
    match File::open(cache_path.clone()).await {
        Err(_) => {
            let client = reqwest::Client::builder().gzip(true).build()?;
            let body = client.get(parsed_url).send().await?;
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
