use crate::config::LOCATION_TTL;
use anyhow::{anyhow, Error};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sunrise::Coordinates;

#[derive(Clone, Copy)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
    pub last_updated: SystemTime,
}


impl Location {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Location {
    fn default() -> Self {
        // Default to Toronto Union Station
        Self {
            lat: 43.64528,
            lon: -79.38056,
            last_updated: SystemTime::UNIX_EPOCH,
        }
    }
}


impl From<&Location> for Coordinates {
    fn from(loc: &Location) -> Self {
        // Clamp lat/lon into valid ranges before constructing Coordinates to avoid panics
        // from out-of-range values that could come from a malformed API or cache response.
        let lat = loc.lat.clamp(-90.0, 90.0);
        let lon = loc.lon.clamp(-180.0, 180.0);
        Coordinates::new(lat, lon).expect("Clamped lat/lon must always be in range")
    }
}

impl Location {
    /// Checks if the current location is still valid (based off the timestamp).
    /// Returns false if the location data is expired or if the clock has skewed backwards.
    pub fn validate(&self) -> bool {
        let elapsed = SystemTime::now()
            .duration_since(self.last_updated)
            .unwrap_or_default()
            .as_secs();
        LOCATION_TTL >= elapsed
    }

    pub fn from_cache() -> Result<Self, Error> {
        // Build the path to ~/.cache/asahi-location-cache
        let mut path = PathBuf::from(env::var("HOME")?);
        path.push(".cache");
        path.push("asahi-location-cache");

        // Open the file
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        // Read lines and parse floats
        let mut lines = reader.lines();
        let lat = lines.next().ok_or(anyhow!("Malformed Cache: Missing Latitude"))??.trim().parse()?;
        let lon = lines.next().ok_or(anyhow!("Malformed Cache: Missing Longitude"))??.trim().parse()?;
        let last_updated = lines.next()
            .and_then(|res| res.ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);

        Ok(Self {
            lat,
            lon,
            last_updated: UNIX_EPOCH.checked_add(Duration::from_secs(last_updated)).unwrap_or(UNIX_EPOCH),
        })
    }

    pub fn to_cache(self) -> Result<(), Error> {
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
        writeln!(&mut file, "{}", self.lat)?;
        writeln!(&mut file, "{}", self.lon)?;

        // `last_updated` as a UNIX timestamp
        let last_updated = self.last_updated.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO).as_secs();
        writeln!(&mut file, "{:?}", last_updated)?;
        Ok(())
    }

}