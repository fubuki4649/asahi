use crate::_utils::tz::iana_to_fixed_offset;
use crate::location::Location;
use chrono::{DateTime, FixedOffset, Local, TimeZone};
use sunrise::{SolarDay, SolarEvent};


#[derive(Debug)]
pub struct SunInfo {
    pub sunrise: DateTime<FixedOffset>,
    pub sunset: DateTime<FixedOffset>,
}

impl Default for SunInfo {
    fn default() -> Self {
        let zero = FixedOffset::east_opt(0).unwrap();
        Self {
            sunrise: zero.timestamp_opt(0, 0).unwrap(),
            sunset: zero.timestamp_opt(0, 0).unwrap(),
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
        let today = SolarDay::new(location.into(), Local::now().date_naive());

        let sunrise_utc = today.event_time(SolarEvent::Sunrise).unwrap_or_default();
        let sunset_utc = today.event_time(SolarEvent::Sunset).unwrap_or_default();

        let sunrise_offset = iana_to_fixed_offset(&location.timezone, sunrise_utc.timestamp());
        let sunset_offset = iana_to_fixed_offset(&location.timezone, sunset_utc.timestamp());

        self.sunrise = sunrise_utc.with_timezone(&sunrise_offset);
        self.sunset = sunset_utc.with_timezone(&sunset_offset);
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