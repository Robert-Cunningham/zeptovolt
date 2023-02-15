use regex::Regex;

use crate::Part;

pub fn configure_redis(con: &mut redis::Connection) {
    match redis::cmd("FT.DROPINDEX").arg("partIndex").query(con) {
        Ok(()) => {}
        Err(_) => {}
    }

    let mut create_index_command = redis::cmd("FT.CREATE");

    for s in Regex::new("[ |\n]").unwrap().split(REDIS_CREATE_INDEX) {
        create_index_command.arg(s.trim());
    }

    match create_index_command.query::<()>(con) {
        Err(e) => println!("{:?}", e),
        Ok(_) => println!("Created index."),
    }
}

pub fn load_into_redis(con: &mut redis::Connection, part: &Part) -> () {
    let json_part = serde_json::to_string(part).unwrap();

    // println!("{}", json_part);

    redis::cmd("JSON.SET")
        .arg(format!("part:{}", part.lcsc_id))
        .arg("$")
        .arg(format!("{}", json_part))
        .query(con)
        .unwrap()
}

pub fn search_redis(con: &mut redis::Connection, string: String) -> Vec<Part> {
    println!("search query {}", string);
    let out: redis::Value = redis::cmd("FT.SEARCH")
        .arg("partIndex")
        .arg(format!(
            "@lcsc_id|image_url|manufacturer_id|basic_or_extended|price|stock:({})",
            string
        ))
        .query(con)
        .unwrap();

    let results = out
        .as_map_iter()
        .unwrap()
        .skip(1)
        .filter_map(|s| {
            let bulk_val = s.0;
            let inner_bulk = match bulk_val {
                redis::Value::Bulk(b) => b,
                _ => panic!("Response wasn't bulk."),
            };
            let raw_json = &inner_bulk[1];
            let json_string = match raw_json {
                redis::Value::Data(v) => std::str::from_utf8(&v).unwrap(),
                _ => panic!(),
            };
            let p: Option<Part> = serde_json::from_str(json_string).ok();

            if p.is_none() {
                eprintln!("failed to parse json string... {}", json_string);
            }
            return p;
        })
        .collect::<Vec<Part>>();

    return results;
}

// PREFIX 1 part:
const REDIS_CREATE_INDEX: &str = "partIndex ON JSON PREFIX 1 part: SCHEMA
$.lcsc_id AS lcsc_id TEXT
$.manufacturer_id AS manufacturer_id TEXT
$.description AS description TEXT
$.image_url AS image_url TEXT
$.basic_or_extended AS basic_or_extended TEXT
$.price AS price NUMERIC
$.stock AS stock NUMERIC";
