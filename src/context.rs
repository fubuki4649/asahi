use crate::config::load_config;
use crate::location::model::Location;
use crate::location::providers::ip::IpLocationProvider;
use crate::location::providers::manual::ManualLocationProvider;
use crate::location::providers::provider_trait::LocationProvider;
use crate::location::providers::wrapper::LocationProviderWrapper;
use chrono::{DateTime, Local, NaiveDate, Utc};
use log::{debug, info, warn};
use sunrise::{SolarDay, SolarEvent};


pub struct Context {
    location_provider: LocationProviderWrapper,
    location: Location,

    // Internal states for current date, and calculated sunrise/sunset times
    date: NaiveDate,
    sunrise: DateTime<Utc>,
    sunset: DateTime<Utc>,

    // Config values controlled by the CLI tool
    /// Manual override for dark mode (-1 = no override);
    pub override_theme: i32,

    // Config values loaded from /etc/asahi/config.toml and ~/.config/asahi/config.toml
    /// How long location data stays valid (seconds). Default: 3600 (1 hour).
    pub location_ttl: u64,
    /// How often to check for sunrise/sunset (seconds). Default: 600 (10 minutes).
    pub sunset_check_frequency: u64,
    /// Minutes to shift when "daytime" begins relative to true sunrise. Negative = earlier.
    pub sunrise_offset: i64,
    /// Minutes to shift when "daytime" ends relative to true sunset. Negative = earlier.
    pub sunset_offset: i64,
}

impl Default for Context {
    fn default() -> Self {
        let cfg = load_config();

        let location_ttl = cfg.get("location_ttl")
            .and_then(toml::Value::as_integer)
            .unwrap_or(3600).cast_unsigned();

        let sunset_check_frequency = cfg.get("sunset_check_frequency")
            .and_then(toml::Value::as_integer)
            .unwrap_or(600).cast_unsigned();

        let sunrise_offset = cfg.get("sunrise_offset")
            .and_then(toml::Value::as_integer)
            .unwrap_or(0);

        let sunset_offset = cfg.get("sunset_offset")
            .and_then(toml::Value::as_integer)
            .unwrap_or(0);

        let lat = cfg.get("override_lat").and_then(toml::Value::as_float);
        let lon = cfg.get("override_lon").and_then(toml::Value::as_float);

        // Init list of location providers
        let providers: Vec<Box<dyn LocationProvider + Send + Sync>> =
            vec![
                Box::new(ManualLocationProvider::new(lat, lon)),
                Box::new(IpLocationProvider)
            ];

        Self {
            location: Location::default(),
            location_provider: LocationProviderWrapper::new(providers),
            date: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            sunrise: Utc::now(),
            sunset: Utc::now(),
            override_theme: -1,
            location_ttl,
            sunset_check_frequency,
            sunrise_offset,
            sunset_offset,
        }
    }
}

impl Context {

    pub fn new() -> Self {
        Self::default()
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
        if !self.location.validate(self.location_ttl) {
            match self.location_provider.get_location() {
                Ok(location) => {
                    self.location = location;
                    self.update_sunrise();
                }
                Err(e) => warn!("Failed to update location, retaining last known location: {e}"),
            }
        }
    }

    pub fn calculate_dark_mode(&mut self) -> u32 {
        if self.override_theme == -1 {
            // Update location/sunrise/sunset times first.
            // Note: update_location() already calls update_sunrise() when location changes,
            // but we call it here too in case the date changed without the location expiring.
            self.update_location();
            self.update_sunrise();
            let now = Utc::now();

            // Apply configured offsets to the true astronomical times.
            // Positive offset = shift later, negative = shift earlier.
            let effective_sunrise = self.sunrise + chrono::Duration::minutes(self.sunrise_offset);
            let effective_sunset  = self.sunset + chrono::Duration::minutes(self.sunset_offset);

            // Send light mode (2) signal if it is daytime
            if effective_sunrise <= now && now < effective_sunset {
                2
            }
            // Otherwise, set dark mode (1) signal
            else {
                1
            }
        } else {
            self.override_theme.cast_unsigned()
        }
    }

    pub fn on_cleanup(&self) {
        self.location_provider.on_cleanup();
    }

    /// Returns the location currently used for sunrise/sunset calculations.
    pub fn location(&self) -> Location {
        self.location
    }

    /// Calculates the timestamp of the next expected sunrise/sunset transition,
    /// taking configured offsets into account. If both of today's transitions
    /// have already passed, this rolls over to tomorrow's sunrise.
    pub fn next_transition_at(&mut self) -> DateTime<Utc> {
        let now = Utc::now();

        let effective_sunrise = self.sunrise + chrono::Duration::minutes(self.sunrise_offset);
        let effective_sunset  = self.sunset + chrono::Duration::minutes(self.sunset_offset);

        if now < effective_sunrise {
            effective_sunrise
        } else if now < effective_sunset {
            effective_sunset
        } else {
            // Both of today's transitions have passed; compute tomorrow's sunrise.
            let tomorrow = self.date.succ_opt().unwrap_or(self.date);
            let tomorrows_times = SolarDay::new((&self.location).into(), tomorrow);
            tomorrows_times.event_time(SolarEvent::Sunrise) + chrono::Duration::minutes(self.sunrise_offset)
        }
    }

}