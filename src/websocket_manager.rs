use anyhow::Result;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::TcpStream;
use std::sync::Arc;
use websocket::sync::stream::TlsStream;
use websocket::sync::Client;
use websocket::{ClientBuilder, Message};

use crate::types::{
  CPUStats, DiskStats, GPUStats, NetworkInterfaceStats, RAMStats, SwapStats, TempStats,
};
use crate::util::arcmutex;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum WebsocketEvent {
  Login {
    auth_token: String,
  },
  DynamicData {
    cpu: CPUStats,
    ram: RAMStats,
    swap: SwapStats,
    gpu: Option<GPUStats>,
    process_count: i32,
    disks: Vec<DiskStats>,
    temps: Option<Vec<TempStats>>,
    network: Vec<NetworkInterfaceStats>,
    host_uptime: u64,
    reporter_uptime: u64,
  },
  StaticData {
    hostname: Option<String>,
    public_ip: Option<String>,
    country: Option<String>,
    city: Option<String>,
    isp: Option<String>,
    timezone: Option<i32>,
    cpu_model: String,
    os_version: Option<String>,
    os_name: Option<String>,
    cpu_cores: Option<usize>,
    cpu_threads: usize,
    total_mem: u64,
    reporter_version: String,
  },
}

pub fn get_event_id(ev: &WebsocketEvent) -> &str {
  match ev {
    WebsocketEvent::Login { .. } => "login",
    WebsocketEvent::StaticData { .. } => "static-data",
    WebsocketEvent::DynamicData { .. } => "dynamic-data",
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebsocketMessage {
  e: WebsocketEvent,
  data: Value,
}

pub struct WebsocketManager {
  pub websocket_url: String,
  pub websocket: Arc<Mutex<Client<TlsStream<TcpStream>>>>,
}

impl WebsocketManager {
  pub fn new(websocket_url: &str) -> Result<Self> {
    let mut client = ClientBuilder::new(websocket_url)?;
    Ok(Self {
      websocket_url: websocket_url.to_string(),
      websocket: arcmutex(client.connect_secure(None)?),
    })
  }

  pub fn send(&mut self, data: WebsocketEvent) -> Result<()> {
    let message = Message::text(
      json!({
          "e": get_event_id(&data),
          "d": &data,
      })
      .to_string(),
    );

    Ok(self.websocket.lock().send_message(&message)?)
  }
}
