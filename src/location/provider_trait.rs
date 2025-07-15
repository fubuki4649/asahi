use crate::location::location::Location;

pub trait LocationProvider {
    /// Attempts to get new location data
    fn get_location() -> Location;
    /// Cleanup hook
    fn on_daemon_close(location: Location);
}