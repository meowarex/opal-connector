use crate::types::NetworkInterfaceStats;
use crate::util::parse_speed;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::str::FromStr;

use std::env;
use sysinfo::{NetworkExt, SystemExt};

#[allow(non_snake_case)] // https://github.com/xornet-cloud/Reporter/pull/24#pullrequestreview-927780254
#[derive(Serialize, Deserialize, Debug)]
pub struct WindowsNetworkInterface {
  pub name: String,
  pub LinkSpeed: String,
}

use super::DataCollector;

impl DataCollector {
  /// Gets the current network stats

  pub fn get_network(&mut self) -> Result<Vec<NetworkInterfaceStats>> {
    let mut nics = Vec::new();

    if self.iterator_index == 0 {
      self.fetcher.refresh_networks_list();
    } else {
      self.fetcher.refresh_networks();
    }

    let nicspeeds = if self.iterator_index == 0 && env::consts::OS == "windows" {
      DataCollector::get_nic_linkspeeds()?
    } else {
      vec![]
    };

    for (interface_name, data) in self.fetcher.networks() {
      // Ignore bullshit loopback interfaces, no one cares
      if interface_name.contains("NPCAP")
        || interface_name.starts_with("lo")
        || interface_name.starts_with("loopback")
      {
        continue;
      };

      if self.iterator_index == 0 {
        // Get the speed of the interface on linux otherwise it's 0
        let speed = match env::consts::OS {
          "linux" => DataCollector::get_nic_linkspeed(interface_name)?,
          "windows" => {
            let nic_index = nicspeeds
              .iter()
              .position(|(name, _)| name == interface_name);

            if let Some(nic_index) = nic_index {
              nicspeeds[nic_index].1
            } else {
              0.0
            }
          }
          _ => 0.0,
        };
        self
          .network_interface_speeds
          .insert(interface_name.to_string(), speed);
      }

      let nic = NetworkInterfaceStats {
        n: interface_name.to_string(),
        tx: data.transmitted() * 8,
        rx: data.received() * 8,
        s: self
          .network_interface_speeds
          .get(&interface_name.to_string())
          .unwrap_or(&0.0)
          .to_owned(),
      };

      nics.push(nic);
    }

    Ok(nics)
  }

  fn get_nic_linkspeed(interface_name: &str) -> Result<f32> {
    let interface_path = format!("/sys/class/net/{}/speed", interface_name);
    let interface_speed = Command::new("cat").arg(&interface_path).output()?;
    let interface_speed =
      f32::from_str(&String::from_utf8_lossy(&interface_speed.stdout).replace("\n", ""))
        .unwrap_or(0.0);
    Ok(interface_speed)
  }

  fn get_nic_linkspeeds() -> Result<Vec<(String, f32)>> {
    let mut nics: Vec<(String, f32)> = Vec::new();
    let output_string = Command::new("powershell")
      .args([
        "-Command",
        "Get-NetAdapter | select name, linkSpeed | ConvertTo-Json",
      ])
      .output()?;

    // Convert the json output to a vector of WindowsNetworkInterface structs
    let output_json = serde_json::from_str::<Vec<WindowsNetworkInterface>>(
      &String::from_utf8_lossy(&output_string.stdout),
    );

    if let Ok(output_json) = output_json {
      output_json.iter().for_each(|nic| {
        let split: Vec<&str> = nic.LinkSpeed.split_whitespace().collect();
        let speed = split[0];
        let mult = split[1];

        nics.push((
          nic.name.to_string(),
          parse_speed(f32::from_str(speed).unwrap_or(0.0), mult),
        ));
      });
    }

    Ok(nics)
  }
}
