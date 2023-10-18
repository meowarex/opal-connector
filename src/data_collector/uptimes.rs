use std::time::SystemTime;

use anyhow::Result;
use sysinfo::SystemExt;

use super::DataCollector;

impl DataCollector {
  /// Get uptime of the system
  pub fn get_uptime(&mut self) -> Result<u64> {
    let boot_time = self.fetcher.boot_time() * 1000;
    let timeframe = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)?
      .as_millis() as u64
      - boot_time;
    Ok(timeframe)
  }

  /// Get uptime of the reporter
  pub fn get_reporter_uptime(&mut self) -> Result<u64> {
    let timeframe = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)?
      .as_millis()
      - self.start_timestamp;
    Ok(timeframe as u64)
  }
}
