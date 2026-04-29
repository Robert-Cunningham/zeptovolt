# Zeptovolt

Zeptovolt is a Rust-backed JLCPCB parts search with full regex support, parallelized scans, and prefix-aware Roaring bitmap caching over a regularly refreshed public parts dataset.

| Query | Cold | Warm | Matches |
|---|---:|---:|---:|
| `0603` | `57.75ms` | `907.92us` | `33,474` |
| `10k` | `43.48ms` | `198.08us` | `6,561` |
| `0603 10k` | `73.32ms` | `79.50us` | `586` |

The backend is a threaded Rust/Axum search service. It downloads the public JLC parts dataset into memory, scans uncached terms in parallel with Rayon, stores each term's matches as a compressed Roaring bitmap, and resolves multi-term searches with fast bitmap intersections.
