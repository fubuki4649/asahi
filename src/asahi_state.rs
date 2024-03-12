use chrono::{Datelike, TimeZone, Utc};
use sunrise::sunrise_sunset;

pub struct AsahiState {

    pub sunrise: i64,
    pub sunset: i64,

    pub longitude: f64,
    pub latitude: f64,

    pub year: i32,
    pub month: u32,
    pub day: u32,

    pub is_dark_mode: bool

}


impl AsahiState {
    
    pub fn new() -> Self {
        Self {
            sunrise: 0,
            sunset: 0,
            longitude: 0.0,
            latitude: 0.0,
            year: 0,
            month: 0,
            day: 0,
            is_dark_mode: false
        }
    }

    pub fn calculate_sunrise(&mut self) {

        let now = Utc::now();

        self.year = now.year();
        self.month = now.month();
        self.day = now.day();

        println!("Date Acquired: {}-{}-{}", self.month, self.day, self.year);

        (self.sunrise, self.sunset) = sunrise_sunset(self.latitude, self.longitude, self.year, self.month, self.day);

        println!("Sunrise: {}, Sunset: {}", Utc.timestamp_opt(self.sunrise, 0).unwrap(), Utc.timestamp_opt(self.sunset, 0).unwrap())

    }

    pub fn update_location(&mut self, latitude: f64, longitude: f64) {

        self.latitude = latitude;
        self.longitude = longitude;

        self.calculate_sunrise();

    }
    
}