use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use server::{download_db, search::search_parts_indexed_with_info};
use std::hint::black_box;

const QUERIES: [&str; 6] = ["0603", "10k", "0603 10k", "300V", "resistor", "resistor "];
const CAPACITOR_PREFIX_QUERIES: [&str; 6] =
    ["ca", "capa", "capac", "capaci", "capacito", "capacitor"];

fn bench_queries(c: &mut Criterion, group_name: &str, queries: &[&str]) {
    let runtime = tokio::runtime::Runtime::new().expect("failed to create Tokio runtime");
    let db = runtime
        .block_on(download_db())
        .expect("failed to download parts database");

    let mut group = c.benchmark_group(group_name);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));

    for query in queries {
        group.bench_with_input(
            BenchmarkId::new("cold", format!("{query:?}")),
            query,
            |b, query| {
                b.iter(|| {
                    db.clear_cache();
                    let result = search_parts_indexed_with_info(black_box(&db), black_box(query));
                    black_box((result.parts.len(), result.parts_searched));
                });
            },
        );

        db.clear_cache();
        let result = search_parts_indexed_with_info(&db, query);
        black_box((result.parts.len(), result.parts_searched));

        group.bench_with_input(
            BenchmarkId::new("cached", format!("{query:?}")),
            query,
            |b, query| {
                b.iter(|| {
                    let result = search_parts_indexed_with_info(black_box(&db), black_box(query));
                    black_box((result.parts.len(), result.parts_searched));
                });
            },
        );
    }

    group.finish();
}

fn bench_search(c: &mut Criterion) {
    bench_queries(c, "search", &QUERIES);
}

fn bench_capacitor_prefix_search(c: &mut Criterion) {
    bench_queries(c, "capacitor_prefix", &CAPACITOR_PREFIX_QUERIES);
}

fn bench_capacitor_prefix_typing_sequence(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().expect("failed to create Tokio runtime");
    let db = runtime
        .block_on(download_db())
        .expect("failed to download parts database");

    let mut group = c.benchmark_group("capacitor_prefix_sequence");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));

    group.bench_function("typing", |b| {
        b.iter(|| {
            db.clear_cache();

            for query in CAPACITOR_PREFIX_QUERIES {
                let result = search_parts_indexed_with_info(black_box(&db), black_box(query));
                black_box((result.parts.len(), result.parts_searched));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_search,
    bench_capacitor_prefix_search,
    bench_capacitor_prefix_typing_sequence
);
criterion_main!(benches);
