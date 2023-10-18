use crate::{data_collector::DataCollectorError, types::TempStats};
use anyhow::{anyhow, Result};
use sysinfo::{ComponentExt, SystemExt};

use super::DataCollector;

impl DataCollector {
  /// Get the current temperature of the system
  pub fn get_temps(&mut self) -> Result<Vec<TempStats>> {
    self.fetcher.refresh_components();

    let components = self.fetcher.components();

    if components.is_empty() {
      return Err(anyhow!(DataCollectorError::NoTemp));
    };

    let mut temps = Vec::<TempStats>::new();
    for component in components {
      let temp = component.temperature();
      temps.push(TempStats {
        label: component.label().to_string(),
        value: temp,
      });
    }
    Ok(temps)
  }
}
