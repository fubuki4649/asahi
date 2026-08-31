use crate::location::providers::ip::IpLocationProvider;
use crate::location::providers::provider_trait::LocationProvider;
use crate::location::providers::wrapper::LocationProviderWrapper;

/// Builds the location provider used by the daemon.
///
/// This returns a [`LocationProviderWrapper`] configured with every available
/// [`LocationProvider`], tried in order until one succeeds. Adding a new
/// location source only requires implementing [`LocationProvider`] and
/// adding it to the list below; no other call sites need to change.
pub fn build_location_provider() -> LocationProviderWrapper {
    let providers: Vec<Box<dyn LocationProvider + Send + Sync>> = vec![Box::new(IpLocationProvider)];
    LocationProviderWrapper::new(providers)
}

/// The period of time for which the location data is valid for (in seconds)
// 3600 seconds = 1 hour
pub const LOCATION_TTL: u64 = 3600;

/// How often to check for sunset
// 600 seconds = 10 minutes
pub const SUNSET_CHECK_FREQUENCY: u64 = 600;