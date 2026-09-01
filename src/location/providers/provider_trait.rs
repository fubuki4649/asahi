use crate::location::model::Location;
use anyhow::Error;

/// A source of location data.
///
/// Implementations are tried at runtime by [`LocationProviderProxy`](crate::location::providers::wrapper::LocationProviderWrapper),
/// which allows the daemon to fall through multiple providers (e.g. IP-based
/// geolocation, a cached location, `GeoClue`, a manual override, ...) instead
/// of being locked into a single, compile-time-selected provider.
pub trait LocationProvider {
    /// Attempts to get new location data. Returns an error if this provider
    /// was unable to determine a location, so that callers (e.g. a proxy
    /// trying multiple providers) can fall back to another source.
    fn get_location(&self) -> Result<Location, Error>;
    /// Cleanup function called when the daemon is shutting down.
    fn on_cleanup(&self);
}