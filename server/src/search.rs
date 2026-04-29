use std::{collections::HashMap, time::Instant};

use indicatif::ProgressIterator;
use regex::Regex;
use roaring::RoaringBitmap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Part {
    pub lcsc_id: String,
    pub manufacturer_id: String,
    pub description: String,
    pub price: f32,
    pub image_url: Option<String>,
    pub datasheet_url: String,
    pub basic_or_extended: String, // todo
    pub stock: u32,
}

enum JLPCBStatus {
    Basic,
    Extended,
    Neither,
}

#[derive(Clone, Debug)]
pub struct PartsDb {
    pub all_parts: Vec<Part>,
    pub cache: HashMap<String, RoaringBitmap>,
    pub last_update: Instant,
}

impl PartsDb {
    fn new() -> PartsDb {
        PartsDb {
            all_parts: Vec::new(),
            cache: HashMap::new(),
            last_update: Instant::now(),
        }
    }
}

pub fn sort_parts(parts: &mut Vec<&Part>) {
    log::debug!("first element {:?}", parts.first());
    parts.sort_unstable_by_key(|x| {
        if x.basic_or_extended == "Basic" {
            -1 * (x.stock as i32)
        } else {
            i32::MAX - x.stock as i32
        }
    });
    log::debug!("first element after sort {:?}", parts.first());
}

pub fn search_parts_indexed<'a>(db: &'a mut PartsDb, string: &String) -> Vec<&'a Part> {
    let words = string
        .split_ascii_whitespace()
        .filter(|w| w.len() >= 2)
        .map(str::to_string)
        .collect::<Vec<_>>();

    log::debug!("search words {:?}", words.len());

    let mut bitmaps = words
        .iter()
        .map(|w| get_match_bitmap(db, w.to_string()).clone());
    let mut out = bitmaps.next().unwrap_or_else(RoaringBitmap::new);

    log::debug!("first {:?}", out.len());

    for bitmap in bitmaps {
        out &= bitmap;
    }

    log::debug!("out {:?}", out.len());

    let mut parts = out
        .iter()
        .filter_map(|i| db.all_parts.get(i as usize))
        .collect::<Vec<_>>();

    log::debug!("before sort first element {:?}", parts.first());
    sort_parts(&mut parts);
    log::debug!("after sort first element {:?}", parts.first());

    log::debug!("parts {:?}", parts.len());

    return parts;
}

fn get_match_bitmap(db: &mut PartsDb, word: String) -> &RoaringBitmap {
    assert!(word.len() >= 2);

    let r = match Regex::new(&format!("(?i){}", word)) {
        Ok(r) => r,
        Err(_) => {
            let escaped = regex::escape(&word);
            Regex::new(&escaped).expect("Escaped regex failed to unwrap?")
        }
    };

    let cached = db.cache.entry(word);

    let after = cached.or_insert_with(|| {
        let does_match = |p: &Part| {
            r.is_match(&p.description)
                || r.is_match(&p.manufacturer_id)
                || r.is_match(&p.basic_or_extended)
                || r.is_match(&p.lcsc_id)
        };

        let mut indexes = RoaringBitmap::new();
        for (i, p) in db.all_parts.iter().enumerate() {
            if does_match(p) {
                indexes.insert(i.try_into().expect("part index exceeded u32"));
            }
        }

        indexes
    });

    return after;
}

fn common_words(db: &mut PartsDb) -> Vec<String> {
    let mut common: HashMap<String, usize> = HashMap::new();

    db.all_parts
        .iter()
        .map(|p| {
            vec![
                &p.description,
                &p.basic_or_extended,
                &p.manufacturer_id,
                &p.lcsc_id,
            ]
        })
        .flatten()
        .map(|s| s.split_ascii_whitespace())
        .flatten()
        .for_each(|w| {
            common
                .entry(w.to_ascii_lowercase())
                .and_modify(|e| *e += 1)
                .or_insert(1);
        });

    common
        .drain()
        .filter_map(|(k, v)| if v > 10000 { Some(k) } else { None })
        .collect::<Vec<_>>()
}

pub fn warm_cache(db: &mut PartsDb) {
    common_words(db)
        .iter()
        .progress()
        .filter(|w| w.len() >= 2)
        .for_each(|w| {
            log::debug!("Analyzing: {}.", w);
            get_match_bitmap(db, w.to_string());
        });
}
