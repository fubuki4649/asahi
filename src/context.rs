use crate::config::SelectedLocationProvider;
use crate::location::location::Location;
use crate::location::provider_trait::LocationProvider;
use chrono::{DateTime, Local, NaiveDate, Utc};
use log::{debug, info};
use sunrise::{SolarDay, SolarEvent};

pub struct Context {
    pub location: Location,

    pub date: NaiveDate,
    pub sunrise: DateTime<Utc>,
    pub sunset: DateTime<Utc>,

    pub manual_darkmode: i32,
}

impl Context {
    
    pub fn new() -> Self {
        Self {
            location: Location::new(),
            date: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            sunrise: Utc::now(),
            sunset: Utc::now(),
            manual_darkmode: -1,
        }
    }

    /// Recalculates the sunrise/sunset times if out of date
    pub fn update_sunrise(&mut self) {
        let today = Local::now().date_naive();

        if self.date != today {
            self.date = today;

            let todays_times = SolarDay::new((&self.location).into(), today);
            self.sunrise = todays_times.event_time(SolarEvent::Sunrise);
            self.sunset = todays_times.event_time(SolarEvent::Sunset);

            info!("Acquired Sunrise/Sunset for {} at lat: {}, lon: {}", today, self.location.lat, self.location.lon);
            debug!("Sunrise: {}, Sunset: {}", self.sunrise, self.sunset);
        }
    }

    /// Recalculates location data if out of date
    pub fn update_location(&mut self) {
        if self.location.validate() == false {
            self.location = SelectedLocationProvider::get_location();
            self.update_sunrise();
        }
    }

    pub fn calculate_dark_mode(&mut self) -> u32 {
        if self.manual_darkmode == -1 {
            // Update location/sunrise/sunset times first
            self.update_location();
            self.update_sunrise();
            let now = Utc::now();

            // Send light mode (2) signal if its daytime
            return if self.sunrise <= now && now < self.sunset {
                2
            }
            // Otherwise, set dark mode (1) signal
            else {
                1
            }
        }

        self.manual_darkmode as u32
    }
    
}