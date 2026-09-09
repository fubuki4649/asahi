use crate::location::Location;
use chrono::{DateTime, Local, TimeZone};
use chrono_tz::Tz;
use sunrise::{SolarDay, SolarEvent};


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
        let today = SolarDay::new((*location).into(), Local::now().date_naive());

        self.sunrise = today
            .event_time(SolarEvent::Sunrise)
            .unwrap_or_default()
            .with_timezone(&location.timezone);
        self.sunset = today
            .event_time(SolarEvent::Sunset)
            .unwrap_or_default()
            .with_timezone(&location.timezone);
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