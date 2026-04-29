use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    download_db,
    search::{search_parts_indexed, Part, PartsDb},
};

#[derive(Clone)]
struct WebServerState {
    //con: Arc<Mutex<redis::Connection>>,
    db: Arc<tokio::sync::RwLock<PartsDb>>,
}

async fn status() -> &'static str {
    return "Ok";
}

const MAX_STALENESS_SECS: u64 = 60 * 60 * 24 * 3;

#[derive(Serialize)]
struct SearchResponse {
    results: Vec<Part>,
    info: SearchInfo,
}

#[derive(Serialize)]
struct SearchInfo {
    parts_searched: usize,
    total_results: usize,
    server_time_ms: u64,
}

#[axum_macros::debug_handler]
async fn search(
    Query(params): Query<HashMap<String, String>>,
    State(wss): State<WebServerState>,
) -> Json<SearchResponse> {
    let q = params.get("q").unwrap().to_string();
    let all_parts = wss.db.read().await;
    let parts_searched = all_parts.all_parts.len();
    let start = std::time::Instant::now();
    let results = search_parts_indexed(&all_parts, &q);
    let server_time = start.elapsed();
    let total_results = results.len();
    log::debug!("Searched for {} in {:?}.", q, server_time);
    let prep = results.into_iter().take(100).cloned().collect::<Vec<_>>();

    return Json(SearchResponse {
        results: prep,
        info: SearchInfo {
            parts_searched,
            total_results,
            server_time_ms: server_time.as_millis().try_into().unwrap_or(u64::MAX),
        },
    });
}

pub async fn webserver(db: PartsDb) {
    let shared_db = Arc::new(tokio::sync::RwLock::new(db));

    let refresh_db_handle = tokio::spawn(refresh_db_periodically(shared_db.clone()));

    let app = Router::new()
        .route("/status", get(status))
        .route("/search", get(search))
        .with_state(WebServerState { db: shared_db })
        .layer(ServiceBuilder::new().layer(CorsLayer::new().allow_origin(Any)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8090));

    log::info!("Serving on 8090...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    // Cancel the refresh_db_periodically task when the server stops
    refresh_db_handle.abort();
}

async fn refresh_db_periodically(db: Arc<tokio::sync::RwLock<PartsDb>>) {
    let refresh_interval = Duration::from_secs(MAX_STALENESS_SECS);
    loop {
        tokio::time::sleep(refresh_interval).await;
        log::info!("Database stale, updating database...");
        let new_db_result = download_db().await;

        match new_db_result {
            Ok(new_db) => {
                let mut db_write_lock = db.write().await;
                *db_write_lock = new_db;
                log::info!("Database updated successfully.");
            }
            Err(e) => {
                log::error!("Failed to update database: {:?}", e);
            }
        }
    }
}
