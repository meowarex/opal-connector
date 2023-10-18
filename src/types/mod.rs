use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StaticData {
  pub hostname: Option<String>,
  pub os_version: Option<String>,
  pub os_name: Option<String>,
  pub cpu_cores: Option<usize>,
  pub public_ip: Option<String>,
  pub isp: Option<String>,
  pub country: Option<String>,
  pub city: Option<String>,
  pub timezone: Option<i32>,
  pub cpu_model: String,
  pub cpu_threads: usize,
  pub total_mem: u64,
  pub reporter_version: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DynamicData {
  pub cpu: CPUStats,
  pub ram: RAMStats,
  pub gpu: Option<GPUStats>,
  pub process_count: i32,
  pub swap: SwapStats,
  pub disks: Vec<DiskStats>,
  pub temps: Option<Vec<TempStats>>,
  pub network: Vec<NetworkInterfaceStats>,
  pub host_uptime: u64,
  pub reporter_uptime: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkInterfaceStats {
  pub n: String,
  pub tx: u64,
  pub rx: u64,
  pub s: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CPUStats {
  pub usage: Vec<u16>,
  pub freq: Vec<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RAMStats {
  pub used: u64,
  pub total: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwapStats {
  pub used: u64,
  pub total: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GPUStats {
  pub brand: String,
  pub gpu_usage: u32,
  pub power_usage: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiskStats {
  pub name: String,
  pub mount: String,
  pub fs: String,
  pub r#type: String,
  pub total: u64,
  pub used: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TempStats {
  pub label: String,
  pub value: f32,
}
