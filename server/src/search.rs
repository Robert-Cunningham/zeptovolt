use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use indicatif::ProgressIterator;
use regex::Regex;
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
    pub cache: HashMap<String, Vec<usize>>,
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
    println!("first element {:?}", parts.first());
    parts.sort_unstable_by_key(|x| {
        if x.basic_or_extended == "Basic" {
            -1 * (x.stock as i32)
        } else {
            i32::MAX - x.stock as i32
        }
    });
    println!("first element after sort {:?}", parts.first());
}

pub fn search_parts_indexed<'a>(db: &'a mut PartsDb, string: &String) -> Vec<&'a Part> {
    let words = string.split_ascii_whitespace().filter(|w| w.len() >= 2);
    let indexes_set: Vec<_> = words
        .map(|w| HashSet::from_iter(get_match_indexes(db, w.to_string()).iter().cloned()))
        .collect();

    println!("is {:?}", indexes_set.len());

    let mut indexes_iter = indexes_set.into_iter();
    let first = indexes_iter.next().unwrap_or_default();

    println!("first {:?}", first.len());

    let out = indexes_iter.fold(first, |set1: HashSet<usize>, set2: HashSet<usize>| {
        set1.intersection(&set2)
            .cloned()
            .collect::<HashSet<usize, _>>()
    });

    println!("out {:?}", out.len());

    let mut parts = out
        .iter()
        .map(|i| db.all_parts.get(*i).unwrap())
        .collect::<Vec<_>>();

    println!("bs first element {:?}", parts.first());
    sort_parts(&mut parts);
    println!("as first element {:?}", parts.first());

    println!("parts {:?}", parts.len());

    return parts;
}

fn get_match_indexes(db: &mut PartsDb, word: String) -> &Vec<usize> {
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

        let indexes = db
            .all_parts
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if does_match(p) {
                    Some(i as usize)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        return indexes;
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
            println!("Analyzing: {}.", w);
            get_match_indexes(db, w.to_string());
        });
}
