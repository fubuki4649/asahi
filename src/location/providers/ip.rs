use crate::location::model::Location;
use crate::location::providers::provider_trait::LocationProvider;
use anyhow::{anyhow, Error};
use log::{debug, info, warn};
use std::thread::sleep;
use std::time::{Duration, SystemTime};

/// Number of additional attempts after the first failure.
const RETRIES: u32 = 3;
/// Delay between attempts.
const RETRY_DELAY: Duration = Duration::from_secs(2);

pub struct IpLocationProvider;

impl IpLocationProvider {

    // Gets the location as (lat, lon)
    fn get_location_ip() -> Result<(f64, f64), Error> {
        let r = minreq::get("https://ip-api.com/json").send()?;
        let response = r.as_str()?;

        // Very naive parsing; uses JSON keys to extract values
        let lat_marker = "\"lat\":";
        let lon_marker = "\"lon\":";

        let lat_idx = response.find(lat_marker).ok_or(anyhow!("Latitude not found"))? + lat_marker.len();
        let lon_idx = response.find(lon_marker).ok_or(anyhow!("Longitude not found"))? + lon_marker.len();

        // parse lat
        let lat_str = &response[lat_idx..];
        let lat_end = lat_str.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-').unwrap_or(lat_str.len());
        let lat: f64 = lat_str[..lat_end].parse()?;

        // parse lon
        let lon_str = &response[lon_idx..];
        let lon_end = lon_str.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-').unwrap_or(lon_str.len());
        let lon: f64 = lon_str[..lon_end].parse()?;

        Ok((lat, lon))
    }
}

impl LocationProvider for IpLocationProvider {
    fn get_location(&self) -> Result<Location, Error> {
        let now = SystemTime::now();
        let total_attempts = RETRIES + 1;
        let mut last_err = None;

        for attempt in 1..=total_attempts {
            match Self::get_location_ip() {
                Ok((lat, lon)) => {
                    debug!("Location acquired by IP: Lat: {lat}, Lon: {lon}");
                    return Ok(Location { lat, lon, last_updated: now });
                }
                Err(e) => {
                    warn!("IP geolocation attempt {attempt}/{total_attempts} failed: {e}");
                    last_err = Some(e);

                    if attempt < total_attempts {
                        info!("Retrying in {}s...", RETRY_DELAY.as_secs());
                        sleep(RETRY_DELAY);
                    }
                }
            }
        }

        let err = last_err.unwrap();
        warn!("IP geolocation failed after all {total_attempts} attempts: {err}");
        Err(err)
    }

    fn on_cleanup(&self) {
        // Nothing to implement here
    }
}