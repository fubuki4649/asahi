use crate::config::LOCATION_TTL;
use std::time::SystemTime;
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
        Self {
            lat: 0.0,
            lon: 0.0,
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
}