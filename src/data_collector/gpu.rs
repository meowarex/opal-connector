use anyhow::Result;
use nvml::NVML;

use crate::types::GPUStats;

use super::{DataCollector, DataCollectorError};

#[derive(Debug)]
pub struct GPUFetcher {
  pub nvidia: Option<NVML>,
}

impl DataCollector {
  /// Get the current GPU states
  pub fn get_gpu(&mut self) -> Result<GPUStats> {
    let gpu_fetcher = &self.gpu_fetcher;
    match gpu_fetcher.nvidia.as_ref() {
      Some(nvml) => {
        let device = nvml.device_by_index(0)?;
        let (brand, util) = (
          format!("{:?}", device.brand()?),
          device.utilization_rates()?,
        );

        Ok(GPUStats {
          brand,
          gpu_usage: util.gpu,
          power_usage: device.power_usage()?,
        })
      }
      None => Err(DataCollectorError::NoGPU.into())
    }
  }
}
