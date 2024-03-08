use tokio::time::{sleep, Duration};
use chrono::{Datelike, Utc};
use sunrise::sunrise_sunset;

pub struct Sunrise {
    
    sunrise: i64,
    sunset: i64,

    pub longitude: f64,
    pub latitude: f64,
    
    year: i32,
    month: u32,
    day: u32,
    
    is_dark_mode: bool,
    toggle_dark_mode: Box<dyn FnMut(bool)>
    
}

impl Sunrise {

    pub fn new(toggle_dark_mode: Box<dyn FnMut(bool)>) -> Self {
        Self {
            sunrise: 0,
            sunset: 0,
            longitude: 0.0,
            latitude: 0.0,
            year: 0,
            month: 0,
            day: 0,
            is_dark_mode: false,
            toggle_dark_mode
        }
    }
    
    pub async fn start(&mut self) {
        
        loop {
            
            let now = Utc::now();
            
            // Check Date and make sure that sunrise/sunset times are for the current day
            if self.year != now.year() || self.month != now.month() || self.day != now.day() {
                
                self.get_today();
                self.update_location(self.latitude, self.longitude);
                
            }
            
            // Check Dark Mode
            // Disable dark mode between sunrise and sunset
            if self.sunrise <= now.timestamp() && now.timestamp() < self.sunset {
                
                if self.is_dark_mode {

                    self.is_dark_mode = false;
                    self.toggle_dark_mode.as_mut()(false);
                    
                }
                
            // Enable dark mode before sunrise/after sunset
            } else if !self.is_dark_mode {
                
                self.is_dark_mode = true;
                self.toggle_dark_mode.as_mut()(true);
                
            }
            
            // Sleep - Only check every minute
            sleep(Duration::from_secs(1)).await;
        }
        
    }
    
    fn get_today(&mut self) {
        
        let now = Utc::now();
        
        self.year = now.year();
        self.month = now.month();
        self.day = now.day();
        
    }
    
    pub fn update_location(&mut self, latitude: f64, longitude: f64) {

        (self.sunrise, self.sunset) = sunrise_sunset(latitude, longitude, self.year, self.month, self.day)
    
    }
    
    
}