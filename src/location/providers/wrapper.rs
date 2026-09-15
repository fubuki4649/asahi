use crate::location::providers::provider_trait::LocationProvider;
use crate::location::types::Location;
use anyhow::{anyhow, Error};
use log::{info, warn};
use std::sync::Mutex;

/// A [`LocationProvider`] that delegates to a list of underlying providers,
/// trying each of them in order until one successfully returns a location.
///
/// This decouples provider *selection* from compile time: instead of the
/// daemon being locked into a single, hard-coded provider type, the proxy is
/// configured at runtime with an ordered list of candidate providers.
/// Currently only [`IpLocationProvider`](crate::location::providers::ip::IpLocationProvider)
/// exists, but adding a new source (e.g. `GeoClue`, GPS, a manual override) is
/// just a matter of implementing [`LocationProvider`] and registering it,
/// without touching any other call sites.
pub struct LocationProviderWrapper {
    providers: Vec<Box<dyn LocationProvider + Send + Sync>>,
    latest_location: Mutex<Location>,
    ttl: u64,
}

impl LocationProviderWrapper {
    pub fn new(providers: Vec<Box<dyn LocationProvider + Send + Sync>>, ttl: u64) -> Self {
        Self {
            providers,
            latest_location: Mutex::new(Location::from_cache().unwrap_or_default()),
            ttl
        }
    }
}

impl LocationProvider for LocationProviderWrapper {
    fn get_location(&self) -> Result<Location, Error> {
        let mut latest_location = self.latest_location.lock().map_err(|err| anyhow!(err.to_string()))?;

        // If the previously calculated location is expired, try to update it
        if !latest_location.validate(self.ttl) {
            // Try each provider in order until one succeeds
            for provider in &self.providers {
                match provider.get_location() {
                    Ok(location) => {
                        *latest_location = location.clone();
                        return Ok(location)
                    },
                    Err(e) => warn!("Location provider failed: {e}"),
                }
            }

            info!("Failed to refresh location. Falling back to cached location");
        }

        // If the last location is still valid,
        // or a new location could not be determined, return the last known location instead
        Ok(latest_location.clone())
    }

    fn on_cleanup(&self) {
        if let Err(e) = self.latest_location.lock().unwrap().to_cache() {
            warn!("Failed to write location cache: {e}");
        }
    }
}
