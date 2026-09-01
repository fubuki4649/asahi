use crate::location::model::Location;
use crate::location::providers::provider_trait::LocationProvider;
use anyhow::{anyhow, Error};
use log::{debug, warn};
use std::time::SystemTime;


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

        match Self::get_location_ip() {
            Ok(location) => {
                debug!("Location acquired by IP: Lat: {}, Lon: {}", location.0, location.1);
                Ok(Location {
                    lat: location.0,
                    lon: location.1,
                    last_updated: now,
                })
            },
            // Otherwise, try reading the cached location
            Err(e) => {
                warn!("Failed to get fresh location via IP: {e}");
                Err(e)
            }
        }
    }

    fn on_cleanup(&self) {
        // Nothing to implement here
    }
}