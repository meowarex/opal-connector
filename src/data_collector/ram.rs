use anyhow::Result;
use sysinfo::SystemExt;

use crate::types::{RAMStats, SwapStats};

use super::DataCollector;

impl DataCollector {
  /// Gets the current RAM stats
  pub fn get_ram(&mut self) -> Result<RAMStats> {
    self.fetcher.refresh_memory();

    Ok(RAMStats {
      used: self.fetcher.used_memory(),
      total: self.fetcher.total_memory(),
    })
  }

  /// Gets the current swap states
  pub fn get_swap(&mut self) -> Result<SwapStats> {
    Ok(SwapStats {
      used: self.fetcher.used_swap(),
      total: self.fetcher.total_swap(),
    })
  }
}
