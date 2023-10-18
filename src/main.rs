use anyhow::Result;
use core::time;
use std::thread;
use std::time::Instant;
use util::arcmutex;

extern crate nvml_wrapper as nvml;

mod arg_parser;
mod auth_manager;
mod config_manager;
mod data_collector;
mod reporter;
mod types;
mod util;
mod websocket_manager;
use crate::reporter::Reporter;

#[tokio::main]
async fn main() -> Result<()> {
  // Create a new instance of the reporter
  let reporter = arcmutex(Reporter::new().await?);

  loop {
    let start_time = Instant::now();

    let fetch_start_time = Instant::now();
    match reporter.lock().update_dynamic_data().await {
      Ok(_) => {}
      Err(e) => {
        println!("{}", e);
        thread::sleep(time::Duration::from_secs(1));
      }
    }
    let fetch_elapsed = fetch_start_time.elapsed();

    let send_start_time = Instant::now();
    match reporter.lock().send_dynamic_data().await {
      Ok(_) => {}
      Err(e) => {
        eprintln!("Error while sending dynamic data: {}", e);
      }
    }
    let send_elapsed = send_start_time.elapsed();

    let total_elapsed = start_time.elapsed();

    let mut rest_time = reporter.lock().args.interval - total_elapsed.as_secs_f64();
    if rest_time < 0.0 {
      rest_time = 0.0;
    }

    println!(
      "Fetch: [{}ms] Send: [{}ms] Total: [{}ms] - Rest: [{}s]",
      fetch_elapsed.as_millis().to_string(),
      send_elapsed.as_millis().to_string(),
      total_elapsed.as_millis().to_string(),
      rest_time.to_string()
    );

    thread::sleep(time::Duration::from_secs_f64(rest_time));
  }
}
