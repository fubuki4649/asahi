use anyhow::{anyhow, Error};
use chrono_tz::Tz;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};


#[derive(Clone, Copy)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
    pub timezone: Tz,
    pub last_updated: SystemTime,
}

impl Default for Location {
    fn default() -> Self {
        // Default to Shinjuku
        Self {
            lat: 35.6887,
            lon: 139.7007,
            timezone: Tz::Japan,
            last_updated: SystemTime::UNIX_EPOCH,
        }
    }
}

impl Location {
    /// Checks if the current location is still valid (based off the timestamp).
    /// Returns false if the location data is expired or if the clock has skewed backwards.
    pub fn validate(&self, ttl: u64) -> bool {
        let elapsed = SystemTime::now()
            .duration_since(self.last_updated)
            .unwrap_or_default()
            .as_secs();
        ttl >= elapsed
    }

    pub fn from_cache() -> Result<Self, Error> {
        // Build the path to ~/.cache/asahi-location-cache
        let path = PathBuf::from(env::var("HOME")?)
            .join(".cache")
            .join("asahi-location-cache");

        // Open the file
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        // Read lines and parse floats
        let mut lines = reader.lines();
        let lat = lines.next().ok_or(anyhow!("Malformed Cache: Missing Latitude"))??.trim().parse()?;
        let lon = lines.next().ok_or(anyhow!("Malformed Cache: Missing Longitude"))??.trim().parse()?;
        let timezone = lines.next().ok_or(anyhow!("Malformed Cache: Missing Timezone"))??.trim().parse().map_err(|e| anyhow!("Malformed Cache: Invalid IANA timezone: {e}"))?;
        let last_updated = lines.next()
            .and_then(Result::ok)
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);

        Ok(Self {
            lat,
            lon,
            timezone,
            last_updated: UNIX_EPOCH.checked_add(Duration::from_secs(last_updated)).unwrap_or(UNIX_EPOCH),
        })
    }

    pub fn to_cache(self) -> Result<(), Error> {
        // Build the path to ~/.cache/asahi-location-cache
        let path = PathBuf::from(env::var("HOME")?)
            .join(".cache")
            .join("asahi-location-cache");

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open file for writing (truncates)
        let mut file = File::create(&path)?;
        writeln!(&mut file, "{}", self.lat)?;
        writeln!(&mut file, "{}", self.lon)?;

        // Write the tz to cache too
        writeln!(&mut file, "{}", self.timezone)?;

        // `last_updated` as a UNIX timestamp
        let last_updated = self.last_updated.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO).as_secs();
        writeln!(&mut file, "{last_updated:?}")?;
        Ok(())
    }

}

impl From<Location> for sunrise::Coordinates {
    fn from(loc: Location) -> Self {
        Self::new(loc.lat, loc.lon).expect("invalid coordinates")
    }
}