mod cpu;
mod disks;
mod geolocation;
mod gpu;
mod nics;
mod ram;
mod temps;
mod uptimes;

use crate::types::{DynamicData, StaticData};
use anyhow::{anyhow, Result};
use nvml::NVML;
use std::{collections::HashMap, time::SystemTime};
use sysinfo::{ProcessRefreshKind, ProcessorExt, System, SystemExt};
use thiserror::Error;

use self::gpu::GPUFetcher;

#[cfg(target_family = "windows")]
use windows::Win32::System::Performance::*;

#[cfg(target_family = "windows")]
use std::ptr;

#[cfg(target_family = "windows")]
use std::thread::sleep;

#[cfg(target_family = "windows")]
use windows::core::PCSTR;

#[cfg(target_family = "windows")]
use windows::Win32::Foundation::ERROR_SUCCESS;

#[cfg(target_family = "windows")]
use std::process::exit;

#[derive(Error, Debug)]
pub enum DataCollectorError {
  #[error("GPU usage unavailable")]
  NoGPU,
  #[error("Temperature unavailable")]
  NoTemp,
}

#[cfg(target_family = "unix")]
#[derive(Debug)]
pub struct DataCollector {
  pub gpu_fetcher: GPUFetcher,
  pub fetcher: System,
  pub program_iterations: usize,
  iterator_index: usize,
  network_interface_speeds: HashMap<String, f32>,
  start_timestamp: u128,
}

#[cfg(target_family = "windows")]
#[derive(Debug)]
pub struct DataCollector {
  pub gpu_fetcher: GPUFetcher,
  pub fetcher: System,
  pub program_iterations: usize,
  pub pdh_query: isize,
  pub pdh_proc_perf_counter: isize,
  pub pdh_proc_freq_counter: isize,
  pub pdh_proc_util_counter: isize,
  pub pdh_proc_perf_data_cached: *mut PDH_FMT_COUNTERVALUE_ITEM_A,
  pub pdh_proc_perf_data_len: usize,
  pub pdh_proc_perf_data_capacity: usize,
  pub pdh_proc_freq_data_cached: *mut PDH_FMT_COUNTERVALUE_ITEM_A,
  pub pdh_proc_freq_data_len: usize,
  pub pdh_proc_freq_data_capacity: usize,
  pub pdh_proc_util_data_cached: *mut PDH_FMT_COUNTERVALUE_ITEM_A,
  pub pdh_proc_util_data_len: usize,
  pub pdh_proc_util_data_capacity: usize,
  pub first_pdh_called: bool,
  iterator_index: usize,
  network_interface_speeds: HashMap<String, f32>,
  start_timestamp: u128
}


impl DataCollector {
  /// Creates a new data collector
  #[cfg(target_family = "unix")]
  pub fn new() -> Result<Self> {
    let (fetcher, gpu_fetcher) = (
      System::new_all(),
      GPUFetcher {
        nvidia: NVML::init().ok(),
      }
    );

    Ok(Self {
      gpu_fetcher,
      fetcher,
      iterator_index: 0,
      program_iterations: 60,
      network_interface_speeds: HashMap::new(),
      start_timestamp: SystemTime::now()
          .duration_since(SystemTime::UNIX_EPOCH)?
          .as_millis(),
    })
  }

  /// Creates a new data collector but initializing Windows centric state.
  #[cfg(target_family = "windows")]
  pub fn new() -> Result<Self> {
    let (fetcher, gpu_fetcher) = (
      System::new_all(),
      GPUFetcher {
        nvidia: NVML::init().ok(),
      }
    );

    let mut pdh_query = 0 as isize;
    let mut pdh_proc_perf_counter = 0 as isize;
    let mut pdh_proc_freq_counter = 0 as isize;
    let mut pdh_proc_util_counter = 0 as isize;

    let pdh_proc_perf_data_cached = ptr::null_mut();
    let pdh_proc_freq_data_cached = ptr::null_mut();
    let pdh_proc_util_data_cached = ptr::null_mut();

    unsafe {
      // Establishes our means to query the winapi performance monitoring system.
      let ret = PdhOpenQueryA(PCSTR::default(), 0, &mut pdh_query as *mut isize);

      if ret != ERROR_SUCCESS.0 as i32 {
        eprintln!("Unable to open perfmon query. Errno: {}", ret);
        exit(ret);
      }

      // Metric to determine per logical CPU speed relative to base clock of CPU.
      // Example: CPU base frequency is 3500 Mhz and processor performance is 106.
      //          Performance value should be interpreted as a percentage, so in this case
      //          base_freq * (performance / 100) = 3710 Mhz.
      let ret = PdhAddEnglishCounterA(pdh_query,
                                      "\\Processor Information(*)\\% Processor Performance",
                                      0,
                                      &mut pdh_proc_perf_counter as *mut isize);

      if ret != ERROR_SUCCESS.0 as i32 {
        eprintln!("Unable to add processor performance counter to perfmon query. Errno: {}", ret);
        exit(ret);
      }

      // Do I really have to say it?
      let ret = PdhAddEnglishCounterA(pdh_query,
                                      "\\Processor Information(*)\\Processor Frequency",
                                      0,
                                      &mut pdh_proc_freq_counter as *mut isize);

      if ret != ERROR_SUCCESS.0 as i32 {
        eprintln!("Unable to add processor frequency counter to perfmon query. Errno: {}", ret);
        exit(ret);
      }

      // Processor utilization per core.
      let ret = PdhAddEnglishCounterA(pdh_query,
                                      "\\Processor Information(*)\\% Processor Utility",
                                      0,
                                      &mut pdh_proc_util_counter as *mut isize);

      if ret != ERROR_SUCCESS.0 as i32 {
        eprintln!("Unable to add processor performance utilization to perfmon query. Errno: {}", ret);
        exit(ret);
      }

      // We have to prime the query data with an initial point of reference. If you
      // don't do this the data retrieved the first time around will either be zeroed or
      // garbage data that won't be useful to you.
      let ret = PdhCollectQueryData(pdh_query);

      if ret != ERROR_SUCCESS.0 as i32 {
        eprintln!("Unable to initialize perfmon queries. Errno: {}", ret);
        exit(ret);
      }

      // Guarantees the next invocation of PdhCollectQueryData will yield valid data.
      sleep(std::time::Duration::from_millis(1000));
    }


    Ok(Self {
      gpu_fetcher,
      fetcher,
      pdh_query,
      pdh_proc_perf_counter,
      pdh_proc_freq_counter,
      pdh_proc_util_counter,
      pdh_proc_perf_data_cached,
      pdh_proc_perf_data_len: 0,
      pdh_proc_perf_data_capacity: 0,
      pdh_proc_freq_data_cached,
      pdh_proc_freq_data_len: 0,
      pdh_proc_freq_data_capacity: 0,
      pdh_proc_util_data_cached,
      pdh_proc_util_data_len: 0,
      pdh_proc_util_data_capacity: 0,
      first_pdh_called: false,
      iterator_index: 0,
      program_iterations: 60,
      network_interface_speeds: HashMap::new(),
      start_timestamp: SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_millis(),
    })
  }

  /// Increments the iterator index by one or resets it to 0 if it reaches the program iterations
  pub fn increment_iterator_index(&mut self) {
    self.iterator_index += 1;
    if self.program_iterations <= self.iterator_index {
      self.iterator_index = 0;
    }
  }

  pub fn get_all_dynamic_data(&mut self) -> Result<DynamicData> {
    #[cfg(target_family = "windows")]
    unsafe {
      let ret = PdhCollectQueryData(self.pdh_query);
      if ret != ERROR_SUCCESS.0 as i32 {
        eprintln!("Unable to query latest CPU information from perfmon.");
        exit(1);
      }
    }

    Ok(DynamicData {
      cpu: self.get_cpu()?,
      ram: self.get_ram()?,
      swap: self.get_swap()?,
      gpu: self.get_gpu().ok(),
      process_count: self.get_total_process_count()? as i32,
      disks: self.get_disks()?,
      temps: self.get_temps().ok(),
      network: self.get_network()?,
      host_uptime: self.get_uptime()?,
      reporter_uptime: self.get_reporter_uptime()?,
    })
  }

  /// Gets the hostname of the system
  pub fn get_hostname() -> Result<String> {
    let fetcher = System::new_all();

    fetcher.host_name().ok_or(anyhow!(
      "Could not get hostname. Are you running this on a supported platform?"
    ))
  }

  /// Gets the total amount of processes running
  pub fn get_total_process_count(&mut self) -> Result<usize> {
    self
      .fetcher
      .refresh_processes_specifics(ProcessRefreshKind::new());
    return Ok(self.fetcher.processes().len());
  }

  /// Gets all the static information about the system
  /// that can't change in runtime
  pub async fn get_statics(&self) -> Result<StaticData> {
    let processor_info = self.fetcher.global_processor_info();
    let geolocation_stuff = DataCollector::get_geolocation_info().await;

    if geolocation_stuff.is_err() {
      return Ok(StaticData {
        cpu_model: processor_info.brand().trim().to_string(),
        public_ip: None,
        country: None,
        isp: None,
        city: None,
        timezone: None,
        hostname: self.fetcher.host_name(),
        os_version: self.fetcher.os_version(),
        os_name: self.fetcher.name(),
        cpu_cores: self.fetcher.physical_core_count(),
        cpu_threads: self.fetcher.processors().len(),
        total_mem: self.fetcher.total_memory(),
        reporter_version: env!("CARGO_PKG_VERSION").to_string(),
      });
    }

    let geolocation_stuff = geolocation_stuff?;

    return Ok(StaticData {
      cpu_model: processor_info.brand().trim().to_string(),
      public_ip: Some(geolocation_stuff.ip),
      country: Some(geolocation_stuff.country_code),
      isp: Some(geolocation_stuff.isp),
      city: Some(geolocation_stuff.city),
      timezone: Some(geolocation_stuff.timezone_gmtOffset),
      hostname: self.fetcher.host_name(),
      os_version: self.fetcher.os_version(),
      os_name: self.fetcher.name(),
      cpu_cores: self.fetcher.physical_core_count(),
      cpu_threads: self.fetcher.processors().len(),
      total_mem: self.fetcher.total_memory(),
      reporter_version: env!("CARGO_PKG_VERSION").to_string(),
    });
  }
}
