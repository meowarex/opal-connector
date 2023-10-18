use crate::arg_parser::ArgParser;
use crate::config_manager::ConfigManager;
use crate::data_collector::DataCollector;
use crate::types::DynamicData;
use crate::websocket_manager::{WebsocketEvent, WebsocketManager};
use anyhow::Result;

pub struct Reporter {
  pub data_collector: DataCollector,
  pub version: String,
  pub config_manager: ConfigManager,
  pub websocket_manager: Option<WebsocketManager>,
  pub args: ArgParser,
  pub dynamic_data: DynamicData,
}

impl Reporter {
  pub async fn new() -> Result<Self> {
    let args = ArgParser::new().await?;
    let websocket_manager: Option<WebsocketManager> = None;

    let config_manager: ConfigManager = ConfigManager::new()?;
    let mut data_collector: DataCollector = DataCollector::new()?;
    let version: String = env!("CARGO_PKG_VERSION").to_string();
    let dynamic_data: DynamicData = data_collector.get_all_dynamic_data()?;

    let mut this = Self {
      data_collector,
      version,
      websocket_manager,
      config_manager,
      args,
      dynamic_data,
    };

    if !this.args.offline {
      this.init_connection()?;
      this.send_static_data().await?;
    }

    Ok(this)
  }

  pub fn init_connection(&mut self) -> Result<()> {
    let websocket_url: String = format!(
      "wss://{}/reporter",
      self.config_manager.config.backend_hostname.to_owned()
    );
    self.websocket_manager = Some(WebsocketManager::new(&websocket_url)?);
    self.login()?;
    Ok(())
  }

  pub fn login(&mut self) -> Result<()> {
    if let Some(websocket_manager) = self.websocket_manager.as_mut() {
      websocket_manager.send(WebsocketEvent::Login {
        auth_token: self.config_manager.config.access_token.to_string(),
      })?;
    }

    Ok(())
  }

  pub async fn send_static_data(&mut self) -> Result<()> {
    if let Some(websocket_manager) = self.websocket_manager.as_mut() {
      let static_data = self.data_collector.get_statics().await?;

      websocket_manager.send(WebsocketEvent::StaticData {
        hostname: static_data.hostname,
        public_ip: static_data.public_ip,
        country: static_data.country,
        city: static_data.city,
        isp: static_data.isp,
        timezone: static_data.timezone,
        cpu_model: static_data.cpu_model,
        os_version: static_data.os_version,
        os_name: static_data.os_name,
        cpu_cores: static_data.cpu_cores,
        cpu_threads: static_data.cpu_threads,
        total_mem: static_data.total_mem,
        reporter_version: self.version.clone(),
      })?;
    }

    Ok(())
  }

  pub async fn update_dynamic_data(&mut self) -> Result<()> {
    self.dynamic_data = self.data_collector.get_all_dynamic_data()?;
    self.data_collector.increment_iterator_index();
    Ok(())
  }

  pub async fn send_dynamic_data(&mut self) -> Result<()> {
    if let Some(websocket_manager) = self.websocket_manager.as_mut() {
      let dd = self.dynamic_data.clone();
      if let Err(e) = websocket_manager.send(WebsocketEvent::DynamicData {
        cpu: dd.cpu,
        ram: dd.ram,
        swap: dd.swap,
        gpu: dd.gpu,
        process_count: dd.process_count,
        disks: dd.disks,
        temps: dd.temps,
        network: dd.network,
        host_uptime: dd.host_uptime,
        reporter_uptime: dd.reporter_uptime,
      }) {
        eprintln!("Websocket error: {}", e);
        self.init_connection()?;
        self.send_static_data().await?;
      }
    }

    Ok(())
  }
}
