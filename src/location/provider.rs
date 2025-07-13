use crate::location::location::Location;

pub trait LocationProvider {
    fn get_location() -> Location;
    fn on_daemon_close(location: Location);
}