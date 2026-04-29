# Zeptovolt

Zeptovolt is a Rust-backed JLCPCB parts search with full regex support, parallelized scans, and prefix-aware Roaring bitmap caching over a regularly refreshed public parts dataset.

It pulls data from [yaqwsx/jlcparts](https://yaqwsx.github.io/jlcparts) and allows searches that are over 10x faster in the benchmarks below.

| Query | Cold | Warm | yaqwsx/jlcparts | Matches |
|---|---:|---:|---:|---:|
| `0603` | `57.75ms` | `907.92us` | `4173ms` | `33,474` |
| `10k` | `43.48ms` | `198.08us` | `4843ms` | `6,561` |
| `0603 10k` | `73.32ms` | `79.50us` | `6155ms` | `586` |

Zeptovolt's listed latencies do not include network time, which will add 100-200ms in practice.

The backend is a threaded Rust/Axum search service. It downloads the public JLC parts dataset into memory, scans uncached terms in parallel with Rayon, stores each term's matches as a compressed Roaring bitmap, and resolves multi-term searches with fast bitmap intersections.
