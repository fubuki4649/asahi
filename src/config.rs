use crate::location::providers::ip::IpLocationProvider;

/// The location provider to use
pub type SelectedLocationProvider = IpLocationProvider;

/// The period of time for which the location data is valid for (in seconds)
// 3600 seconds = 1 hour
pub const LOCATION_TTL: u64 = 3600;

/// How often to check for sunset
// 600 seconds = 10 minutes
pub const SUNSET_CHECK_FREQUENCY: u64 = 600;