use anyhow::Result;
use sysinfo::{DiskExt, SystemExt};

use crate::types::DiskStats;

use super::DataCollector;

impl DataCollector {
  /// Gets the current disk(s) stats
  pub fn get_disks(&mut self) -> Result<Vec<DiskStats>> {
    let mut disks = Vec::<DiskStats>::new();
    self.fetcher.refresh_disks_list();
    for disk in self.fetcher.disks() {
      let (name, mount) = (
        disk.name().to_string_lossy(),
        disk.mount_point().to_string_lossy(),
      );

      if name.contains("docker") || mount.contains("docker") || mount.contains("boot") {
        continue;
      }

      let (fs_type, mut str) = (disk.file_system(), String::from(""));

      for unit in fs_type {
        str.push(*unit as char);
      }

      let disk = DiskStats {
        name: format!("{}", disk.name().to_string_lossy()),
        mount: format!("{}", disk.mount_point().to_string_lossy()),
        fs: str,
        r#type: format!("{:?}", disk.type_()),
        total: disk.total_space(),
        used: disk.total_space() - disk.available_space(),
      };

      disks.push(disk);
    }
    Ok(disks)
  }
}
