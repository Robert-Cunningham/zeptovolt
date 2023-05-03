use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    download_db,
    memorysearch::{search_parts_indexed, PartsDb},
    textsearch::search_redis,
    Part,
};

#[derive(Clone)]
struct WebServerState {
    //con: Arc<Mutex<redis::Connection>>,
    db: Arc<tokio::sync::Mutex<PartsDb>>,
}

async fn status() -> &'static str {
    return "Ok";
}

const MAX_STALENESS_SECS: u64 = 60 * 60 * 24 * 3;

#[axum_macros::debug_handler]
async fn search(
    Query(params): Query<HashMap<String, String>>,
    State(wss): State<WebServerState>,
) -> Json<Vec<Part>> {
    let q = params.get("q").unwrap().to_string();
    // let mut c = wss.con.lock().unwrap();
    // let out = search_redis(&mut c, q);
    let mut all_parts = wss.db.lock().await;
    let start = Instant::now();
    let results = search_parts_indexed(&mut all_parts, &q);
    println!("Searched for {} in {:?}.", q, start.elapsed());
    let prep = results.into_iter().take(100).cloned().collect::<Vec<_>>();

    return Json(prep);
}

pub async fn webserver(db: PartsDb) {
    let shared_db = Arc::new(tokio::sync::Mutex::new(db));

    // Start a task to refresh the database periodically
    let refresh_db_handle = tokio::spawn(refresh_db_periodically(shared_db.clone()));

    let app = Router::new()
        .route("/status", get(status))
        .route("/search", get(search))
        .with_state(WebServerState { db: shared_db })
        .layer(ServiceBuilder::new().layer(CorsLayer::new().allow_origin(Any)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8090));

    println!("Serving...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    // Cancel the refresh_db_periodically task when the server stops
    refresh_db_handle.abort();
}

async fn refresh_db_periodically(db: Arc<tokio::sync::Mutex<PartsDb>>) {
    let refresh_interval = Duration::from_secs(MAX_STALENESS_SECS);
    loop {
        tokio::time::sleep(refresh_interval).await;
        match download_db().await {
            Ok(new_db) => {
                let mut db_write_lock = db.lock().await;
                *db_write_lock = new_db;
                println!("Database updated successfully.");
            }
            Err(e) => {
                eprintln!("Failed to update database: {:?}", e);
            }
        }
    }
}

/*
pub async fn webserver<'a>(db: PartsDb) {
    let app = Router::new()
        .route("/status", get(status))
        .route("/search", get(search))
        .with_state(WebServerState {
            db: Arc::new(tokio::sync::Mutex::new(db)),
        })
        .layer(ServiceBuilder::new().layer(CorsLayer::new().allow_origin(Any)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8090));

    println!("Serving...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
*/

/*
    let since_update = Instant::now()
        .duration_since(all_parts.last_update)
        .as_secs();

    let random_ns = Instant::now()
        .duration_since(all_parts.last_update)
        .as_nanos()
        % 1000;

    if since_update > MAX_STALENESS_SECS && random_ns == 0 {
        match download_db().await {
            Ok(new_parts) => {
                *all_parts = new_parts;
            }
            Err(e) => {
                println!("Error downloading db: {:?}", e);
            }
        }
    }

*/
