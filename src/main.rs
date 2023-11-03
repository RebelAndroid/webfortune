#![feature(div_duration)]
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::routing::get;
use axum::Router;
use csv::{StringRecord, Error};
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::{random, thread_rng};

struct Fortune {
    fortune: String,
    attribution: String,
}

struct AppState {
    current_fortune: usize,
    start_up_time: Instant,
    last_updated_time_slice: u64,
    fortune_list: Vec<Fortune>,
}

#[tokio::main]
async fn main() {
    let mut reader = csv::Reader::from_path("fortunes.csv").unwrap();

    let fortune_list: Vec<Fortune> = reader.records().map(|record| {
        let record = record.unwrap();
        Fortune {
            fortune: record.get(0).unwrap().to_string(),
            attribution: record.get(1).unwrap().to_string(),
        }
    }).collect();

    println!("fortune list length: {}", fortune_list.len());

    let app_state = Arc::new(Mutex::new(AppState {
        current_fortune: 0,
        start_up_time: Instant::now(),
        last_updated_time_slice: 0,
        fortune_list,
    }));
    let app = Router::new().route("/", get(handler)).with_state(app_state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(State(state): State<Arc<Mutex<AppState>>>) -> String {
    let mut state = state.lock().unwrap();
    let current_time = Instant::now();
    let time_slice = current_time
        .duration_since(state.start_up_time)
        .div_duration_f64(Duration::new(5, 0)) as u64;
    if time_slice != state.last_updated_time_slice {
        state.current_fortune = Uniform::new(0, state.fortune_list.len()).sample(&mut thread_rng());
        state.last_updated_time_slice = time_slice;
    }
    let current_fortune = state.fortune_list.get(state.current_fortune).unwrap();
    format!(
        "\"{}\"\n\t-{}",
        current_fortune.fortune, current_fortune.attribution
    )
}
