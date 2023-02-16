use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use indicatif::ProgressIterator;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

use crate::{textsearch::search_redis, Part};

pub struct PartsDb {
    pub all_parts: Vec<Part>,
    pub cache: HashMap<String, Vec<usize>>,
}

impl PartsDb {
    fn new() -> PartsDb {
        PartsDb {
            all_parts: Vec::new(),
            cache: HashMap::new(),
        }
    }
}

/*
pub fn search_parts_direct<'a>(db: &'a PartsDb, string: &String) -> Vec<&'a Part> {
    let words = string.split_ascii_whitespace();
    let regexes: Vec<_> = words
        .map(|w| Regex::new(&format!("(?i){}", w)).unwrap())
        .collect();

    let does_match = |p: &Part| {
        regexes.iter().all(|r| {
            r.is_match(&p.description)
                || r.is_match(&p.manufacturer_id)
                || r.is_match(&p.lcsc_id)
                || r.is_match(&p.basic_or_extended)
        })
    };

    let mut out = db
        .all_parts
        .par_iter()
        .filter(|p| does_match(p))
        .collect::<Vec<_>>();

    sort_parts(&mut out);

    return out;
}
*/

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

    // let indexes_set = indexes
    //     .iter()
    //     .map(|v| HashSet::from_iter(v.iter().cloned()))
    //     .collect::<Vec<_>>();

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
    let r = Regex::new(&format!("(?i){}", word)).unwrap();

    let cached = db.cache.entry(word);

    let after = cached.or_insert_with(|| {
        let does_match = |p: &Part| {
            r.is_match(&p.description)
                || r.is_match(&p.manufacturer_id)
            // || r.is_match(&p.lcsc_id)
            || r.is_match(&p.basic_or_extended)
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
