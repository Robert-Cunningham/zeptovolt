use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
    time::Instant,
};

use anyhow::{anyhow, Context, Result};
use futures::{stream, StreamExt};
use serde_json::Value;

use crate::{
    search::{Part, PartsDb},
    utils::{cached_get, cleanup_old_cache_dirs},
};

const JLC_DATA_BASE_URL: &str = "https://yaqwsx.github.io/jlcparts/data";
const DOWNLOAD_CONCURRENCY: usize = 16;

type AttributeLookup = Vec<(String, Value)>;

fn data_url(file_name: &str) -> String {
    format!("{}/{}", JLC_DATA_BASE_URL, file_name)
}

fn value_at<'a>(
    row: &'a [Value],
    header: &HashMap<String, usize>,
    field: &str,
) -> Result<&'a Value> {
    let index = header
        .get(field)
        .with_context(|| format!("component shard is missing {field} field"))?;

    row.get(*index)
        .with_context(|| format!("component row is missing {field} value"))
}

fn string_at(row: &[Value], header: &HashMap<String, usize>, field: &str) -> Result<String> {
    value_at(row, header, field)?
        .as_str()
        .map(String::from)
        .with_context(|| format!("component {field} value is not a string"))
}

fn optional_string_at(
    row: &[Value],
    header: &HashMap<String, usize>,
    field: &str,
) -> Result<Option<String>> {
    let value = value_at(row, header, field)?;
    if value.is_null() {
        Ok(None)
    } else {
        value
            .as_str()
            .map(|value| Some(value.to_string()))
            .with_context(|| format!("component {field} value is not a string or null"))
    }
}

fn default_attribute_value(attribute: &Value) -> Option<String> {
    attribute
        .get("values")?
        .get("default")?
        .get(0)
        .and_then(|value| match value {
            Value::String(value) => Some(value.clone()),
            Value::Bool(value) => Some(value.to_string()),
            Value::Number(value) => Some(value.to_string()),
            _ => None,
        })
}

fn basic_or_extended(
    row: &[Value],
    header: &HashMap<String, usize>,
    attributes: &AttributeLookup,
) -> Result<String> {
    let attribute_ids = value_at(row, header, "attributes")?
        .as_array()
        .context("component attributes value is not an array")?;

    for attribute_id in attribute_ids {
        let attribute_index = match attribute_id.as_u64() {
            Some(attribute_index) => attribute_index,
            None => continue,
        };
        let (name, attribute) = match attributes.get(attribute_index as usize) {
            Some(attribute) => attribute,
            None => continue,
        };
        if name == "Basic/Extended" {
            return Ok(default_attribute_value(attribute).unwrap_or_else(|| "Unknown".to_string()));
        }
    }

    Ok("Unknown".to_string())
}

fn price_at(row: &[Value], header: &HashMap<String, usize>) -> Result<f32> {
    let price = value_at(row, header, "price")?
        .as_array()
        .and_then(|prices| prices.first())
        .and_then(|price_break| price_break.get("price"))
        .and_then(Value::as_f64)
        .unwrap_or_default();

    Ok(price as f32)
}

fn stock_at(row: &[Value], header: &HashMap<String, usize>) -> Result<u32> {
    let stock = value_at(row, header, "stock")?
        .as_u64()
        .context("component stock value is not an unsigned integer")?;

    Ok(stock.min(u32::MAX as u64) as u32)
}

fn parse_header(line: &str) -> Result<HashMap<String, usize>> {
    let header: HashMap<String, usize> =
        serde_json::from_str(line).context("failed to parse component shard header")?;

    Ok(header)
}

fn parse_component(
    line: &str,
    header: &HashMap<String, usize>,
    attributes: &AttributeLookup,
) -> Result<Part> {
    let row: Vec<Value> = serde_json::from_str(line).context("failed to parse component row")?;

    Ok(Part {
        lcsc_id: string_at(&row, header, "lcsc")?,
        manufacturer_id: string_at(&row, header, "mfr")?,
        description: string_at(&row, header, "description")?,
        price: price_at(&row, header)?,
        image_url: optional_string_at(&row, header, "img")?,
        datasheet_url: optional_string_at(&row, header, "datasheet")?.unwrap_or_default(),
        basic_or_extended: basic_or_extended(&row, header, attributes)?,
        stock: stock_at(&row, header)?,
    })
}

pub async fn process_shard(
    shard_name: String,
    attributes: Arc<AttributeLookup>,
) -> Result<Vec<Part>> {
    let text = cached_get(data_url(&shard_name))
        .await
        .with_context(|| format!("failed to download component shard {shard_name}"))?;
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header_line = lines
        .next()
        .with_context(|| format!("component shard {shard_name} was empty"))?;
    let header = parse_header(header_line)
        .with_context(|| format!("failed to parse component shard {shard_name} header"))?;

    lines
        .enumerate()
        .map(|(index, line)| {
            parse_component(line, &header, &attributes).with_context(|| {
                format!(
                    "failed to parse component row {} in shard {shard_name}",
                    index + 2
                )
            })
        })
        .collect()
}

fn component_shards(manifest: &Value) -> Result<Vec<String>> {
    let categories = manifest
        .get("categories")
        .and_then(Value::as_array)
        .context("manifest is missing categories array")?;

    let mut seen = HashSet::new();
    let mut shards = Vec::new();

    for category in categories {
        let category_shards = category
            .get("shards")
            .and_then(Value::as_array)
            .context("manifest category is missing shards array")?;

        for shard in category_shards {
            let shard = shard
                .as_str()
                .context("manifest category shard is not a string")?
                .to_string();

            if seen.insert(shard.clone()) {
                shards.push(shard);
            }
        }
    }

    Ok(shards)
}

async fn get_manifest() -> Result<Value> {
    let text = cached_get(data_url("manifest.json"))
        .await
        .context("failed to download upstream manifest")?;
    let manifest = serde_json::from_str(&text).context("failed to parse upstream manifest")?;

    Ok(manifest)
}

async fn get_attributes(manifest: &Value) -> Result<AttributeLookup> {
    let attributes_lut = manifest
        .get("attributesLut")
        .and_then(Value::as_str)
        .context("manifest is missing attributesLut")?;
    let text = cached_get(data_url(attributes_lut))
        .await
        .context("failed to download attributes lookup")?;
    let attributes =
        serde_json::from_str(&text).context("failed to parse attributes lookup table")?;

    Ok(attributes)
}

pub async fn download_db() -> Result<PartsDb, anyhow::Error> {
    let manifest = get_manifest().await?;
    let shards = component_shards(&manifest)?;
    let attributes = Arc::new(get_attributes(&manifest).await?);

    log::info!("Downloading {} component shards...", shards.len());

    let results = stream::iter(shards.into_iter().map(|shard| {
        let attributes = attributes.clone();
        async move { process_shard(shard, attributes).await }
    }))
    .buffer_unordered(DOWNLOAD_CONCURRENCY)
    .collect::<Vec<_>>()
    .await;

    log::info!("Done.");

    let mut all_parts = Vec::new();
    for result in results {
        all_parts.extend(result?);
    }

    if all_parts.is_empty() {
        return Err(anyhow!("component download completed with no parts"));
    }

    let db = PartsDb {
        all_parts: all_parts,
        cache: RwLock::new(HashMap::new()),
        last_update: Instant::now(),
    };

    cleanup_old_cache_dirs()?;

    return Ok(db);
}
