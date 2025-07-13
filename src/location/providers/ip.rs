use crate::location::location::Location;
use crate::location::provider::LocationProvider;
use anyhow::{anyhow, Error};
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use log::{debug, info, warn};

pub struct IpLocationProvider;


impl IpLocationProvider {
    pub fn new() -> Self { Self }

    // Gets the location as (lat, lon)
    fn get_location_ip() -> Result<(f64, f64), Error> {
        let r = minreq::get("http://ip-api.com/json").send()?;
        let response = r.as_str()?;

        // Very naive parsing; uses JSON keys to extract values
        let lat_marker = "\"lat\":";
        let lon_marker = "\"lon\":";

        let lat_idx = response.find(lat_marker).ok_or(anyhow!("Latitude not found"))? + lat_marker.len();
        let lon_idx = response.find(lon_marker).ok_or(anyhow!("Longitude not found"))? + lon_marker.len();

        // parse lat
        let lat_str = &response[lat_idx..];
        let lat_end = lat_str.find(|c: char| !c.is_digit(10) && c != '.' && c != '-').unwrap_or(lat_str.len());
        let lat: f64 = lat_str[..lat_end].parse()?;

        // parse lon
        let lon_str = &response[lon_idx..];
        let lon_end = lon_str.find(|c: char| !c.is_digit(10) && c != '.' && c != '-').unwrap_or(lon_str.len());
        let lon: f64 = lon_str[..lon_end].parse()?;

        Ok((lat, lon))
    }

    // Gets the last known location from local cache as (lat, lon)
    fn get_location_cache() -> Result<(f64, f64), Error> {
        // Build the path to ~/.cache/asahi-location-cache
        let mut path = PathBuf::from(env::var("HOME")?);
        path.push(".cache");
        path.push("asahi-location-cache");

        // Open the file
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        // Read lines and parse floats
        let mut lines = reader.lines();
        let lat_line = lines.next().ok_or(anyhow!("Malformed Cache: Missing Latitude"))??;
        let lon_line = lines.next().ok_or(anyhow!("Malformed Cache: Missing Longitude"))??;

        Ok((lon_line.trim().parse()?, lat_line.trim().parse()?))
    }

    // Write to the local location cache
    fn save_location_cache(lat: f64, lon: f64) -> Result<(), Error> {
        // Build the path to ~/.cache/asahi-location-cache
        let mut path = PathBuf::from(env::var("HOME")?);
        path.push(".cache");
        path.push("asahi-location-cache");

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open file for writing (truncates)
        let mut file = File::create(&path)?;
        writeln!(&mut file, "{lat}")?;
        writeln!(&mut file, "{lon}")?;
        Ok(())
    }
}

impl LocationProvider for IpLocationProvider {
    fn get_location() -> Location {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();

        // Try IP location first
        match Self::get_location_ip() {
            Ok(location) => {
                debug!("Lat: {}, Lon: {}", location.0, location.1);
                Location {
                    lon: location.0,
                    lat: location.1,
                    last_updated: now,
                }
            },
            // Otherwise, try reading the cached location
            Err(_) => {
                info!("Unable to get fresh location, using cached value");
                
                match Self::get_location_cache() {
                    Ok(location) => {
                        debug!("Lat: {}, Lon: {}", location.0, location.1);
                        Location {
                            lat: location.0,
                            lon: location.1,
                            last_updated: now,
                        }
                    },
                    // Last resort, assume we are at Toronto Union Station
                    Err(_) => {
                        info!("Unable to read cached location, assuming Toronto Union Station");
                        
                        Location {
                            lat: 43.644444,
                            lon: -79.374722,
                            last_updated: now,
                        }
                    }
                }
            }
        }
    }

    fn on_daemon_close(location: Location) {
        if let Err(e) = Self::save_location_cache(location.lat, location.lon) {
            warn!("Failed to write location cache: {}", e);
        }
    }
}