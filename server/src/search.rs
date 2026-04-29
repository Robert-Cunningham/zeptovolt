use std::{collections::HashMap, sync::RwLock, time::Instant};

use indicatif::ProgressIterator;
use rayon::prelude::*;
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

#[derive(Debug)]
pub struct PartsDb {
    pub all_parts: Vec<Part>,
    pub cache: RwLock<HashMap<String, RoaringBitmap>>,
    pub last_update: Instant,
}

#[derive(Debug)]
pub struct SearchPartsResult<'a> {
    pub parts: Vec<&'a Part>,
    pub terms: Vec<SearchTermInfo>,
    pub parts_searched: usize,
}

#[derive(Debug)]
pub struct SearchTermInfo {
    pub term: String,
    pub cached: bool,
}

impl SearchTermInfo {
    pub fn cache_status(&self) -> &'static str {
        if self.cached {
            "cached"
        } else {
            "uncached"
        }
    }
}

struct MatchBitmap {
    bitmap: RoaringBitmap,
    cached: bool,
}

impl PartsDb {
    fn new() -> PartsDb {
        PartsDb {
            all_parts: Vec::new(),
            cache: RwLock::new(HashMap::new()),
            last_update: Instant::now(),
        }
    }

    pub fn clear_cache(&self) {
        self.cache
            .write()
            .expect("parts search cache lock poisoned")
            .clear();
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

pub fn search_parts_indexed<'a>(db: &'a PartsDb, string: &str) -> Vec<&'a Part> {
    search_parts_indexed_with_info(db, string).parts
}

pub fn search_parts_indexed_with_info<'a>(db: &'a PartsDb, string: &str) -> SearchPartsResult<'a> {
    let words = string
        .split_ascii_whitespace()
        .filter(|w| w.len() >= 2)
        .map(str::to_string)
        .collect::<Vec<_>>();

    log::debug!("search words {:?}", words.len());

    let mut terms = Vec::with_capacity(words.len());
    let mut bitmaps = Vec::with_capacity(words.len());
    let mut parts_searched = 0usize;
    for word in words {
        let match_bitmap = get_match_bitmap(db, &word);
        if !match_bitmap.cached {
            parts_searched = parts_searched.saturating_add(db.all_parts.len());
        }
        terms.push(SearchTermInfo {
            term: word,
            cached: match_bitmap.cached,
        });
        bitmaps.push(match_bitmap.bitmap);
    }

    let mut bitmaps = bitmaps.into_iter();
    let mut out = bitmaps.next().unwrap_or_else(RoaringBitmap::new);

    log::debug!("first {:?}", out.len());

    for bitmap in bitmaps {
        out &= bitmap;
    }

    log::debug!("out {:?}", out.len());

    return parts_result_from_bitmap(db, out, terms, parts_searched);
}

fn parts_result_from_bitmap<'a>(
    db: &'a PartsDb,
    out: RoaringBitmap,
    terms: Vec<SearchTermInfo>,
    parts_searched: usize,
) -> SearchPartsResult<'a> {
    let mut parts = out
        .iter()
        .filter_map(|i| db.all_parts.get(i as usize))
        .collect::<Vec<_>>();

    log::debug!("before sort first element {:?}", parts.first());
    sort_parts(&mut parts);
    log::debug!("after sort first element {:?}", parts.first());

    log::debug!("parts {:?}", parts.len());

    return SearchPartsResult {
        parts,
        terms,
        parts_searched,
    };
}

fn get_cached_match_bitmap(db: &PartsDb, word: &str) -> Option<RoaringBitmap> {
    db.cache
        .read()
        .expect("parts search cache lock poisoned")
        .get(word)
        .cloned()
}

fn compile_match_regex(word: &str) -> Regex {
    match Regex::new(&format!("(?i){}", word)) {
        Ok(r) => r,
        Err(_) => {
            let escaped = regex::escape(&word);
            Regex::new(&escaped).expect("Escaped regex failed to unwrap?")
        }
    }
}

fn part_matches(r: &Regex, p: &Part) -> bool {
    r.is_match(&p.description)
        || r.is_match(&p.manufacturer_id)
        || r.is_match(&p.basic_or_extended)
        || r.is_match(&p.lcsc_id)
}

fn get_match_bitmap_for_all_parts_parallel(db: &PartsDb, word: &str) -> RoaringBitmap {
    let r = compile_match_regex(word);

    db.all_parts
        .par_iter()
        .enumerate()
        .fold(RoaringBitmap::new, |mut indexes, (i, p)| {
            if part_matches(&r, p) {
                indexes.insert(u32::try_from(i).expect("part index exceeded u32"));
            }

            indexes
        })
        .reduce(RoaringBitmap::new, |mut left, right| {
            left |= right;
            left
        })
}

fn save_match_bitmap(db: &PartsDb, word: &str, indexes: RoaringBitmap) -> RoaringBitmap {
    let mut cache = db.cache.write().expect("parts search cache lock poisoned");
    let cached = cache.entry(word.to_string()).or_insert_with(|| indexes);

    cached.clone()
}

fn get_match_bitmap(db: &PartsDb, word: &str) -> MatchBitmap {
    assert!(word.len() >= 2);

    if let Some(cached) = get_cached_match_bitmap(db, word) {
        return MatchBitmap {
            bitmap: cached,
            cached: true,
        };
    }

    let indexes = get_match_bitmap_for_all_parts_parallel(db, word);
    let cached = save_match_bitmap(db, word, indexes);

    return MatchBitmap {
        bitmap: cached,
        cached: false,
    };
}

fn common_words(db: &PartsDb) -> Vec<String> {
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

pub fn warm_cache(db: &PartsDb) {
    common_words(db)
        .iter()
        .progress()
        .filter(|w| w.len() >= 2)
        .for_each(|w| {
            log::debug!("Analyzing: {}.", w);
            get_match_bitmap(db, w);
        });
}
