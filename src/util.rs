use parking_lot::Mutex;
use std::sync::Arc;

pub fn arcmutex<T>(item: T) -> Arc<Mutex<T>> {
  Arc::new(Mutex::new(item))
}

/// Returns the speed in megabytes per second
/// # Arguments
/// * `number` - The number to convert
/// * `speed` - The speed multiplier of the number
pub fn parse_speed(number: f32, speed: &str) -> f32 {
  match speed {
    "bps" => number / 1000000f32,
    "Kbps" => number / 1000f32,
    "Mbps" => number,
    "Gbps" => number * 1000f32,
    "Tbps" => number * 1000000f32,
    _ => number,
  }
}