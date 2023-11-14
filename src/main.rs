#![feature(div_duration)]
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use csv::ReaderBuilder;
use ini::Ini;
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::thread_rng;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct Fortune {
    /// the actual text of the fortune
    fortune: String,
    /// the attribution for the quote, if the quote comes from a piece of media, the authors name should be used
    attribution: String,
    /// the work the quote is from (if applicable)
    work: Option<String>,
    /// character attribution
    character_attribution: Option<String>,
}

struct AppState {
    current_fortune: usize,
    start_up_time: Instant,
    last_updated_time_slice: u64,
    fortune_list: Vec<Fortune>,
    duration: Duration,
}

#[tokio::main]
async fn main() {
    let configuration =
        Ini::load_from_file("configuration.ini").expect("expected file configuration.ini");
    let configuration = configuration
        .section::<String>(None)
        .expect("invalid configuration.ini, all options should be in the top level section");

    let fortunes_path = configuration
        .get("fortunes_path")
        .expect("expected fortunes_path in configuration.ini");
    let mut reader = ReaderBuilder::new()
        .comment(Some(b'#'))
        .flexible(true)
        .has_headers(true)
        .from_path(fortunes_path).expect("expected fortunes_path in configuration.ini to point to a valid csv file");

    let fortune_list: Vec<Fortune> = reader
        .records()
        .map(|record| {
            let record = record.unwrap();
            Fortune {
                fortune: record.get(0).unwrap().trim().to_string(),
                attribution: record.get(1).unwrap().trim().to_string(),
                work: record.get(2).map(|string| string.trim().to_string()),
                character_attribution: record.get(3).map(|string| string.trim().to_string()),
            }
        })
        .collect();

    let secs: u64 = configuration
        .get("time_slice_seconds")
        .expect("expected time_slice_seconds in configuration.ini")
        .parse()
        .expect("expected time_slice_seconds in configuration.ini to be an integer");
    let duration = Duration::new(secs, 0);

    let app_state = Arc::new(Mutex::new(AppState {
        current_fortune: 0,
        start_up_time: Instant::now(),
        last_updated_time_slice: 0,
        fortune_list,
        duration,
    }));
    let app = Router::new()
        .route("/fortune", get(handler))
        .with_state(app_state);

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[axum::debug_handler]
async fn handler(State(state): State<Arc<Mutex<AppState>>>) -> Json<Fortune> {
    let mut state = state.lock().unwrap();
    let current_time = Instant::now();
    let time_slice = current_time
        .duration_since(state.start_up_time)
        .div_duration_f64(state.duration) as u64;
    if time_slice != state.last_updated_time_slice {
        state.current_fortune = Uniform::new(0, state.fortune_list.len()).sample(&mut thread_rng());
        state.last_updated_time_slice = time_slice;
    }
    let current_fortune = state.fortune_list.get(state.current_fortune).unwrap();
    Json(current_fortune.clone())
}
