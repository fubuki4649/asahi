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

    /// Extracts a float value for `key` from a JSON string, tolerating any
    /// amount of whitespace between the key, colon, and value.
    fn extract_float(response: &str, key: &str) -> Result<f64, Error> {
        let key_pattern = format!("\"{}\"", key);
        let key_pos = response
            .find(key_pattern.as_str())
            .ok_or_else(|| anyhow!("{key} not found in response"))?;

        // Advance past the closing quote of the key name, then find the colon.
        let after_key = &response[key_pos + key_pattern.len()..];
        let colon_pos = after_key
            .find(':')
            .ok_or_else(|| anyhow!("{key}: colon not found"))?;

        // Skip the colon and any leading whitespace before the number.
        let after_colon = after_key[colon_pos + 1..].trim_start();

        let end = after_colon
            .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
            .unwrap_or(after_colon.len());

        after_colon[..end]
            .parse::<f64>()
            .map_err(|e| anyhow!("{key}: failed to parse value: {e}"))
    }

    fn get_location_ip() -> Result<(f64, f64), Error> {
        let r = minreq::get("http://ip-api.com/json").send()?;
        let response = r.as_str()?;

        let lat = Self::extract_float(response, "lat")?;
        let lon = Self::extract_float(response, "lon")?;

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