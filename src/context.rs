use crate::config::{load_config, Value};
use crate::location::providers::ip::IpLocationProvider;
use crate::location::providers::manual::ManualLocationProvider;
use crate::location::providers::provider_trait::LocationProvider;
use crate::location::providers::wrapper::LocationProviderWrapper;
use crate::location::Location;
use crate::sun::sun_info::SunInfo;
use crate::sun::sun_stats::SunStats;
use chrono::Local;
use log::{debug, info, warn};

pub struct Context {
    location_provider: LocationProviderWrapper,
    location: Location,

    // Internal states for current date, and calculated sunrise/sunset times
    // sun_stats is `Override(i32)` if there's a manual override in place
    pub sun_stats: SunStats,

    // Config values loaded from /etc/asahi/config.toml and ~/.config/asahi/config.toml
    /// How long location data stays valid (seconds). Default: 3600 (1 hour).
    pub location_ttl: u64,
    /// How often to check for sunrise/sunset (seconds). Default: 600 (10 minutes).
    pub sunset_check_frequency: u64,
}

impl Default for Context {
    fn default() -> Self {
        let cfg = load_config();

        let location_ttl = cfg.get("location_ttl")
            .and_then(Value::as_integer)
            .unwrap_or(3600).cast_unsigned();

        let sunset_check_frequency = cfg.get("sunset_check_frequency")
            .and_then(Value::as_integer)
            .unwrap_or(600).cast_unsigned();

        let lat = cfg.get("override_lat").and_then(Value::as_float);
        let lon = cfg.get("override_lon").and_then(Value::as_float);

        // Init list of location providers
        let providers: Vec<Box<dyn LocationProvider + Send + Sync>> =
            vec![
                Box::new(ManualLocationProvider::new(lat, lon)),
                Box::new(IpLocationProvider)
            ];

        Self {
            location: Location::default(),
            location_provider: LocationProviderWrapper::new(providers),
            sun_stats: SunStats::Calculated(SunInfo::default()),
            location_ttl,
            sunset_check_frequency,
        }
    }
}

impl Context {

    pub fn new() -> Self {
        Self::default()
    }

    /// Recalculates the sunrise/sunset times if out of date
    fn update_sunrise(&mut self) {
        // Only update if there's no manual override present
        if let SunStats::Calculated(stats) = &mut self.sun_stats {
            let now = Local::now().naive_local();

            // If we're on a new day, update sunrise/sunset readings
            if now.date() > stats.sunset.naive_local().date() {
                stats.update(&self.location);
                info!("Updated Sunrise/Sunset for {} at lat: {}, lon: {}", now, self.location.lat, self.location.lon);
                debug!("Sunrise: {}, Sunset: {}", stats.sunrise, stats.sunset);
            }
        }
    }

    /// Recalculates location data if out of date
    fn update_location(&mut self) {
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
        if let SunStats::Override(mode) = self.sun_stats {
            return mode.cast_unsigned();
        }

        // Make sure everything's still fresh
        self.update_location();
        self.update_sunrise();

        // We only care about local time
        if let SunStats::Calculated(ref stats) = self.sun_stats {
            stats.calculate_theme()
        } else {
            unreachable!()
        }
    }

    pub fn set_theme_override(&mut self, mode: i32) {
        if mode == -1 {
            self.sun_stats = SunStats::Calculated(SunInfo::new(&self.location));
        } else {
            self.sun_stats = SunStats::Override(mode);
        }
    }

    pub fn on_cleanup(&self) {
        self.location_provider.on_cleanup();
    }

    /// Returns the location currently used for sunrise/sunset calculations.
    pub fn location(&self) -> Location {
        self.location
    }

}