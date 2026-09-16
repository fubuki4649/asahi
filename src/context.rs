use crate::config::{load_config, Value};
use crate::location::providers::{IpLocationProvider, LocationProvider, LocationProviderWrapper, ManualLocationProvider};
use crate::location::Location;
use crate::sun::{SunInfo, SunStats};
use chrono::{Local, Timelike};
use log::{debug, info};
use std::cmp::min;
use std::time::{Duration, SystemTime};


pub struct Context {
    location_provider: LocationProviderWrapper,
    location: Location,
    location_ttl: u64,

    // Internal states for current date, and calculated sunrise/sunset times
    // sun_stats is `Override(i32)` if there's a manual override in place
    pub sun_stats: SunStats,
}

impl Default for Context {
    fn default() -> Self {
        let cfg = load_config();

        let location_ttl = cfg.get("location_ttl")
            .and_then(Value::as_integer)
            .unwrap_or(3600).cast_unsigned();

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
            location_provider: LocationProviderWrapper::new(providers, location_ttl),
            sun_stats: SunStats::Calculated(SunInfo::default()),
            location_ttl,
        }
    }
}

impl Context {

    pub fn new() -> Self {
        Self::default()
    }

    /// Recalculates the sunrise/sunset times if out of date
    ///
    /// `force_recalc` - Forces recalculation of today's sunrise/sunset times; typically useful
    /// after the location has been updated too
    fn update_sunrise(&mut self, force_recalc: bool) {
        // Only update if there's no manual override present
        if let SunStats::Calculated(stats) = &mut self.sun_stats {
            let now = Local::now().naive_local();

            // If we're on a new day, update sunrise/sunset readings
            if force_recalc || now.date() > stats.sunset.naive_local().date() {
                stats.update(&self.location);
                info!("Updated Sunrise/Sunset for {} at lat: {}, lon: {}", now, self.location.lat, self.location.lon);
                debug!("Sunrise: {}, Sunset: {}", stats.sunrise, stats.sunset);
            }
        }
    }

    /// Recalculates location data if out of date
    pub fn update_location(&mut self) {
        if let Ok(location) = self.location_provider.get_location() {
            self.location = location;
        }
    }

    /// `force_recalc` - Forces recalculation of today's sunrise/sunset times; typically useful
    /// after the location has been updated too
    pub fn calculate_dark_mode(&mut self, force_recalc: bool) -> u32 {
        if let SunStats::Override(mode) = self.sun_stats {
            return mode.cast_unsigned();
        }

        // Make sure the sunrise/sunset times are still OK
        self.update_sunrise(force_recalc);

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
        self.location.clone()
    }

    /// Returns how long to sleep until the next relevant event:
    /// min(time until location TTL expiry, time until next solar event or end of day) + 1 minute.
    pub fn next_wakeup(&self) -> Duration {
        let now = Local::now();
        const SECS_PER_DAY: u64 = 86_400;

        let till_midnight = SECS_PER_DAY - u64::from(now.num_seconds_from_midnight());
        let till_location_ttl = self.location_ttl.saturating_sub(
            SystemTime::now().duration_since(self.location.last_updated).unwrap_or_default().as_secs()
        );
        let till_solar_event = if let SunStats::Calculated(ref info) = self.sun_stats {
            [info.sunrise, info.sunset]
                .iter()
                .filter_map(|t| u64::try_from(t.timestamp() - now.timestamp()).ok())
                .min()
                .unwrap_or(till_midnight)
        } else {
            till_midnight
        };

        Duration::from_secs(min(till_location_ttl, till_solar_event) + 60)
    }

}