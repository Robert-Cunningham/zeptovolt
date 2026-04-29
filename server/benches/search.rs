use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use server::{download_db, search::search_parts_indexed_with_info};
use std::hint::black_box;

const QUERIES: [&str; 6] = ["0603", "10k", "0603 10k", "300V", "resistor", "resistor "];

fn bench_search(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().expect("failed to create Tokio runtime");
    let db = runtime
        .block_on(download_db())
        .expect("failed to download parts database");

    let mut group = c.benchmark_group("search");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));

    for query in QUERIES {
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

criterion_group!(benches, bench_search);
criterion_main!(benches);
