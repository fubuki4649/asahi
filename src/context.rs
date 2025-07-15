use crate::config::SelectedLocationProvider;
use crate::location::location::Location;
use crate::location::provider_trait::LocationProvider;
use chrono::{DateTime, Local, NaiveDate, Utc};
use log::info;
use sunrise::{SolarDay, SolarEvent};

pub struct Context {
    pub location: Location,

    pub date: NaiveDate,
    pub sunrise: DateTime<Utc>,
    pub sunset: DateTime<Utc>,
}

impl Context {
    
    pub fn new() -> Self {
        Self {
            location: Location::new(),
            date: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            sunrise: Utc::now(),
            sunset: Utc::now(),
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

            info!("Acquired Sunrise/Sunset for {}", today);
            info!("Sunrise: {}, Sunset: {}", self.sunrise, self.sunset);
        }
    }

    /// Recalculates location data if out of date
    pub fn update_location(&mut self) {
        if self.location.validate() == false {
            self.location = SelectedLocationProvider::get_location();
            self.update_sunrise();
        }
    }
    
}