use crate::location::Location;
use chrono::{DateTime, Local, TimeZone};
use chrono_tz::Tz;
use std::time::{SystemTime, UNIX_EPOCH};
use sun::SunPhase::{Sunrise, Sunset};


#[derive(Debug)]
pub struct SunInfo {
    pub sunrise: DateTime<Tz>,
    pub sunset: DateTime<Tz>,
}

impl Default for SunInfo {
    fn default() -> Self {
        Self {
            sunrise: Tz::Japan.timestamp_opt(0, 0).unwrap(),
            sunset: Tz::Japan.timestamp_opt(0, 0).unwrap(),
        }
    }
}

impl SunInfo {
    pub fn new(location: &Location) -> Self {
        let mut stats = Self::default();
        stats.update(location);
        stats
    }

    pub fn update(&mut self, location: &Location) {
        let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

        let sunrise_ms = sun::time_at_phase(now_ms as i64, Sunrise, location.lat, location.lon, 0.0);
        let sunset_ms  = sun::time_at_phase(now_ms as i64, Sunset, location.lat, location.lon, 0.0);

        self.sunrise = DateTime::from_timestamp_millis(sunrise_ms).unwrap_or_default().with_timezone(&location.timezone);
        self.sunset = DateTime::from_timestamp_millis(sunset_ms).unwrap_or_default().with_timezone(&location.timezone);
    }

    pub fn calculate_theme(&self) -> u32 {
        let now = Local::now().naive_local();
        let sunrise = self.sunrise.naive_local();
        let sunset = self.sunset.naive_local();

        if sunrise <= now && now < sunset {
            2
        } else {
            1
        }
    }
}