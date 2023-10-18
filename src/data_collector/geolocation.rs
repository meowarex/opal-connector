use super::DataCollector;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

const GEOLOCATION_URL: &str = "https://ipwhois.app/json/";

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct GeolocationInfo {
  pub ip: String,
  pub country_code: String,
  pub isp: String,
  pub city: String,
  pub timezone_gmtOffset: i32,
}

impl DataCollector {
  /// Gets the geolocation information
  pub async fn get_geolocation_info() -> Result<GeolocationInfo> {
    let response = reqwest::get(GEOLOCATION_URL).await?;

    if response.status() == reqwest::StatusCode::OK {
      let geolocation_info: GeolocationInfo = response.json().await?;
      return Ok(geolocation_info);
    } else {
      return Err(anyhow!("Could not get geolocation info"));
    }
  }
}
