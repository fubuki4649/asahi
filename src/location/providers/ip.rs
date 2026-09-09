use crate::_utils::tz::validate_iana_timezone;
use crate::location::providers::provider_trait::LocationProvider;
use crate::location::types::Location;
use anyhow::{anyhow, Error};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use tinyjson::JsonValue;

/// Number of additional attempts after the first failure.
const RETRIES: u32 = 3;
/// Delay between attempts.
const RETRY_DELAY: Duration = Duration::from_secs(2);

pub struct IpLocationProvider;

impl IpLocationProvider {
    fn get_location_ip() -> Result<(f64, f64, String), Error> {
        let r = minreq::get("http://ip-api.com/json").send()?;
        if !(200..300).contains(&r.status_code) { return Err(anyhow!("IP Geolocation Server Error ({})", r.status_code)); }

        let response = r.as_str()?;

        let parsed: HashMap<String, JsonValue> = response
            .parse::<JsonValue>()?
            .try_into()
            .map_err(|_| anyhow!("Failed to parse JSON response from IP geolocation server!"))?;

        let lat = *parsed.get("lat").and_then(JsonValue::get::<f64>).ok_or_else(|| anyhow!("Latitude missing from response"))?;
        let lon = *parsed.get("lon").and_then(JsonValue::get::<f64>).ok_or_else(|| anyhow!("Longitude missing from response"))?;
        let iana_tz = parsed.get("timezone").and_then(JsonValue::get::<String>).ok_or_else(|| anyhow!("Timezone missing from response"))?.as_str();
        if !validate_iana_timezone(iana_tz) {
            return Err(anyhow!("Invalid IANA timezone: {iana_tz}"));
        }

        Ok((lat, lon, iana_tz.to_owned()))
    }
}

impl LocationProvider for IpLocationProvider {
    fn get_location(&self) -> Result<Location, Error> {
        let now = SystemTime::now();
        let total_attempts = RETRIES + 1;
        let mut last_err = None;

        for attempt in 1..=total_attempts {
            match Self::get_location_ip() {
                Ok((lat, lon, timezone)) => {
                    debug!("Location acquired by IP: Lat: {lat}, Lon: {lon}");
                    return Ok(Location {
                        lat,
                        lon,
                        timezone,
                        last_updated: now,
                    });
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