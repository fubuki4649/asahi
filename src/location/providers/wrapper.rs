use crate::location::model::Location;
use crate::location::providers::provider_trait::LocationProvider;
use anyhow::Error;
use log::{info, warn};
use std::cell::Cell;

/// A [`LocationProvider`] that delegates to a list of underlying providers,
/// trying each of them in order until one successfully returns a location.
///
/// This decouples provider *selection* from compile time: instead of the
/// daemon being locked into a single, hard-coded provider type, the proxy is
/// configured at runtime with an ordered list of candidate providers.
/// Currently only [`IpLocationProvider`](crate::location::providers::ip::IpLocationProvider)
/// exists, but adding a new source (e.g. GeoClue, GPS, a manual override) is
/// just a matter of implementing [`LocationProvider`] and registering it,
/// without touching any other call sites.
pub struct LocationProviderWrapper {
    providers: Vec<Box<dyn LocationProvider + Send + Sync>>,
    latest_location: Cell<Location>,
}

impl LocationProviderWrapper {
    pub fn new(providers: Vec<Box<dyn LocationProvider + Send + Sync>>) -> Self {
        Self {
            providers,
            latest_location: Cell::new(Location::from_cache().unwrap_or_default()),
        }
    }
}

impl LocationProvider for LocationProviderWrapper {
    fn get_location(&self) -> Result<Location, Error> {
        // Try each provider in order until one succeeds
        for provider in &self.providers {
            match provider.get_location() {
                Ok(location) => {
                    self.latest_location.set(location);
                    return Ok(location)
                },
                Err(e) => warn!("Location provider failed: {e}"),
            }
        }

        info!("Failed to refresh location. Falling back to cached location");

        // If nothing works, check for a previously cached location
        Ok(self.latest_location.get())
    }

    fn on_cleanup(&self) {
        if let Err(e) = self.latest_location.get().to_cache() {
            warn!("Failed to write location cache: {}", e);
        }
    }
}
