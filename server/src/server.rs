use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};

use crate::{textsearch::search_redis, Part};

#[derive(Clone)]
struct WebServerState {
    con: Arc<Mutex<redis::Connection>>,
}

pub async fn webserver(con: redis::Connection) {
    let app = Router::new()
        .route("/status", get(status))
        .route("/search", get(search))
        .with_state(WebServerState {
            con: Arc::new(Mutex::new(con)),
        });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

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
    let mut c = wss.con.lock().unwrap();
    let out = search_redis(&mut c, q);

    return Json(out);
}
