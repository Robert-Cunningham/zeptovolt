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
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

use crate::{textsearch::search_redis, Part};

#[derive(Clone)]
struct WebServerState {
    //con: Arc<Mutex<redis::Connection>>,
    db: Arc<Mutex<Vec<Part>>>,
}

pub async fn webserver<'a>(/*con: redis::Connection,*/ db: Vec<Part>) {
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
    let all_parts = wss.db.lock().unwrap();
    let results = search_parts(&all_parts, q);
    let prep = results.into_iter().take(100).cloned().collect::<Vec<_>>();

    return Json(prep);
}

fn search_parts(db: &Vec<Part>, string: String) -> Vec<&Part> {
    let words = string.split_ascii_whitespace();
    let regexes: Vec<_> = words
        .map(|w| {
            let regex_pattern = format!("(?i){}", w);
            println!("{}", regex_pattern);
            let r = Regex::new(&regex_pattern).unwrap();
            return r;
        })
        .collect();

    let does_match = |p: &Part| {
        regexes.par_iter().all(|r| {
            r.is_match(&p.description)
                || r.is_match(&p.manufacturer_id)
                || r.is_match(&p.lcsc_id)
                || r.is_match(&p.basic_or_extended)
        })
    };

    let mut out = db.iter().filter(|p| does_match(p)).collect::<Vec<_>>();

    out.sort_unstable_by_key(|x| {
        -1 * (if x.basic_or_extended == "Basic" {
            i32::MAX
        } else {
            x.stock as i32
        })
    });
    return out;
}
