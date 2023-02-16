use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Instant,
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
    memorysearch::{search_parts_indexed, PartsDb},
    textsearch::search_redis,
    Part,
};

#[derive(Clone)]
struct WebServerState {
    //con: Arc<Mutex<redis::Connection>>,
    db: Arc<Mutex<PartsDb>>,
}

pub async fn webserver<'a>(db: PartsDb) {
    let app = Router::new()
        .route("/status", get(status))
        .route("/search", get(search))
        .with_state(WebServerState {
            //con: Arc::new(Mutex::new(con)),
            db: Arc::new(Mutex::new(db)),
        })
        .layer(ServiceBuilder::new().layer(CorsLayer::new().allow_origin(Any)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8090));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn status() -> &'static str {
    return "Ok";
}

async fn search(
    Query(params): Query<HashMap<String, String>>,
    State(wss): State<WebServerState>,
) -> Json<Vec<Part>> {
    let q = params.get("q").unwrap().to_string();
    // let mut c = wss.con.lock().unwrap();
    // let out = search_redis(&mut c, q);
    let mut all_parts = wss.db.lock().unwrap();
    let start = Instant::now();
    let results = search_parts_indexed(&mut all_parts, &q);
    println!("Searched for {} in {:?}.", q, start.elapsed());
    let prep = results.into_iter().take(100).cloned().collect::<Vec<_>>();

    return Json(prep);
}
