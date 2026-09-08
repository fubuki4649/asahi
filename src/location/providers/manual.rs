use crate::location::types::Location;
use crate::location::providers::provider_trait::LocationProvider;
use anyhow::{anyhow, Error};
use chrono_tz::Tz;
use std::fs;
use std::time::SystemTime;

pub struct ManualLocationProvider {
    lat: Option<f64>,
    lon: Option<f64>,
}

impl ManualLocationProvider {
    pub fn new(lat: Option<f64>, lon: Option<f64>) -> Self {
        Self { lat, lon }
    }

    fn system_timezone() -> Result<Tz, Error> {
        let symlink_zone = fs::read_link("/etc/localtime")
            .ok()
            .and_then(|link| link.to_str().map(str::to_owned))
            .and_then(|s| {
                let zone = s.rsplit("zoneinfo/").next()?.to_owned();
                (zone != s).then_some(zone)
            });

        // Try reading the symlink to /etc/timezone, otherwise read /etc/timezone directly on systems like debian
        let iana_tz = match symlink_zone {
            Some(zone) => zone,
            None => fs::read_to_string("/etc/timezone")?.trim().to_owned(),
        };

        iana_tz.parse::<Tz>().map_err(|e| anyhow!("Invalid IANA timezone {iana_tz}: {e}"))
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
                timezone: Self::system_timezone()?,
                last_updated: SystemTime::now(),
            })
        } else {
            Err(anyhow!("Manual location not set"))
        }
    }

    fn on_cleanup(&self) {
    }
}