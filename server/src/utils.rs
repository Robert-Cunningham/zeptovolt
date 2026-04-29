use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use bytes::Buf;
use flate2::read::GzDecoder;
use reqwest::Url;
use std::io::Read;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

use anyhow::Result;

pub fn cleanup_old_cache_dirs() -> Result<()> {
    let entries = match fs::read_dir("cache") {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    let mut cache_dirs = Vec::new();

    for entry in entries {
        let entry = entry?;

        if !entry.file_type()?.is_dir() {
            continue;
        }

        let file_name = entry.file_name();
        if let Some(file_name) = file_name.to_str() {
            if let Ok(cache_day) = file_name.parse::<u64>() {
                cache_dirs.push((cache_day, entry.path()));
            }
        }
    }

    cache_dirs.sort_unstable_by_key(|(cache_day, _)| *cache_day);
    for (_, path) in cache_dirs.into_iter().rev().skip(5) {
        fs::remove_dir_all(path)?;
    }

    Ok(())
}

pub async fn cached_get(url: String) -> Result<String> {
    let day = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() / 86400;
    let day_str = format!("{}", day);

    let parsed_url = Url::parse(&url).expect("malformed url");
    let cache_path_str = vec!["cache", &day_str, &parsed_url.path().replace("/", "-")].join("/");
    let cache_path = PathBuf::from(cache_path_str);
    fs::create_dir_all(cache_path.parent().unwrap())?;

    match File::open(cache_path.clone()).await {
        Err(_) => {
            let client = reqwest::Client::builder().gzip(true).build()?;
            let body = client
                .get(parsed_url.clone())
                .send()
                .await?
                .error_for_status()?;
            let mut write_file = File::create(cache_path).await?;

            if parsed_url.path().ends_with(".gz") {
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
