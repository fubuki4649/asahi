use crate::location::model::Location;
use crate::location::providers::provider_trait::LocationProvider;
use anyhow::{anyhow, Error};
use std::time::SystemTime;


pub struct ManualLocationProvider {
    lat: Option<f64>,
    lon: Option<f64>,
}

impl ManualLocationProvider {
    pub fn new(lat: Option<f64>, lon: Option<f64>) -> Self {
        Self { lat, lon }
    }
}

impl LocationProvider for ManualLocationProvider {
    fn get_location(&self) -> Result<Location, Error> {
        // Return the manually set location, if any
        // If no location is set, this fails and the wrapper moves onto the next option
        if let (Some(lat), Some(lon)) = (self.lat, self.lon) {
            Ok(Location {
                lat,
                lon,
                last_updated: SystemTime::now(),
            })
        } else {
            Err(anyhow!("Manual location not set"))
        }
    }

    fn on_cleanup(&self) {
    }
}