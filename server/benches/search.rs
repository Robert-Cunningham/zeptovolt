use std::time::{Duration, Instant};

use server::{
    download_db,
    search::{search_parts_indexed, PartsDb},
};

const QUERIES: [&str; 3] = ["0603", "10k", "0603 10k"];

fn time_search(db: &mut PartsDb, query: &String) -> (Duration, usize) {
    let started = Instant::now();
    let result_count = search_parts_indexed(db, query).len();

    (started.elapsed(), result_count)
}

fn print_timing(label: &str, query: &str, duration: Duration, result_count: usize) {
    println!(
        "{label:>5} {query:<9} {:>8.2?} ({result_count} matches)",
        duration
    );
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().expect("failed to create Tokio runtime");
    let mut db = runtime
        .block_on(download_db())
        .expect("failed to download parts database");

    println!("Loaded {} parts.", db.all_parts.len());
    println!("Search timings:");

    for query in QUERIES {
        let query_string = query.to_string();

        db.cache.clear();
        let (cold_duration, cold_count) = time_search(&mut db, &query_string);
        print_timing("cold", query, cold_duration, cold_count);

        let (warm_duration, warm_count) = time_search(&mut db, &query_string);
        print_timing("warm", query, warm_duration, warm_count);
    }
}
