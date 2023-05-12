use std::{collections::HashMap, time::Instant};

use anyhow::Result;
use futures::StreamExt;
use serde_json::Value;

use crate::{
    search::{Part, PartsDb},
    utils::cached_get,
};

pub async fn process_category(s: String) -> Result<Vec<Part>> {
    let stock_info = serde_json::from_str(
        &cached_get(format!(
            "https://yaqwsx.github.io/jlcparts/data/{}.stock.json",
            s
        ))
        .await?,
    )?;

    let stock_map = match stock_info {
        Value::Object(m) => m,
        _ => todo!(),
    };

    let part_info: Value = serde_json::from_str(
        &cached_get(format!(
            "https://yaqwsx.github.io/jlcparts/data/{}.json.gz",
            s
        ))
        .await?,
    )?;

    let part_list = match &part_info["components"] {
        Value::Array(a) => a,
        _ => todo!(),
    };

    let out: Vec<_> = part_list
        .iter()
        .map(|p| Part {
            lcsc_id: p[0].as_str().unwrap().into(),
            manufacturer_id: p[1].as_str().unwrap().into(),
            description: p[3].as_str().unwrap().into(),
            price: p[5][0]["price"].as_f64().unwrap() as f32,
            image_url: p[6].as_str().map(|x| x.into()),
            datasheet_url: p[4].as_str().unwrap().to_string(),
            basic_or_extended: p[8]["Basic/Extended"]["values"]["default"][0]
                .as_str()
                .unwrap()
                .into(),
            stock: stock_map
                .get(p[0].as_str().expect("part number wasn't a string?"))
                .expect(format!("stock json didnt have info on part {}", p[0]).as_str())
                .as_i64()
                .unwrap() as u32,
        })
        .collect();

    Ok(out)
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

pub async fn get_categories() -> Result<Vec<String>> {
    let out = cached_get(String::from(
        "https://yaqwsx.github.io/jlcparts/data/index.json",
    ))
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

pub async fn download_db() -> Result<PartsDb, anyhow::Error> {
    let sources = &get_categories().await?;
    let results =
        futures::future::join_all(sources.iter().map(|s| process_category(s.to_string()))).await;

    println!("Done.");

    let all_parts = results
        .iter()
        .flatten()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();

    let db = PartsDb {
        all_parts: all_parts,
        cache: HashMap::new(),
        last_update: Instant::now(),
    };

    return Ok(db);
}
