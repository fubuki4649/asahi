use crate::config::LOCATION_TTL;
use std::time::SystemTime;
use sunrise::Coordinates;

pub struct Location {
    pub lat: f64,
    pub lon: f64,
    pub last_updated: SystemTime,
}


impl Location {
    pub fn new() -> Self {
        Self {
            lat: 0.0,
            lon: 0.0,
            last_updated: SystemTime::UNIX_EPOCH,
        }
    }
}


impl From<&Location> for Coordinates {
    fn from(loc: &Location) -> Self {
        Coordinates::new(loc.lat, loc.lon).unwrap()
    }
}

impl Location {
    /// Checks if the current location is still valid (based off the timestamp)
    pub fn validate(&self) -> bool {
        LOCATION_TTL >= SystemTime::now().duration_since(self.last_updated).expect("Data is from the future").as_secs()
    }
}