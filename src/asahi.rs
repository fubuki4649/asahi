use std::error::Error;

use chrono::{Datelike, Utc};
use sunrise::sunrise_sunset;
use tokio::time::{Duration, sleep};
use zbus::connection;
use zbus::zvariant::Value::U32;

use crate::portal::Settings;

pub struct Asahi {
    
    sunrise: i64,
    sunset: i64,

    pub longitude: f64,
    pub latitude: f64,
    
    year: i32,
    month: u32,
    day: u32,
    
    is_dark_mode: bool
    
}

impl Asahi {

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
    
    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {

        let conn = connection::Builder::session()?
            .name("org.freedesktop.impl.portal.desktop.asahi")?
            .serve_at("/org/freedesktop/portal/desktop", Settings::new())?
            .build().await?;

        let iface_ref = conn
            .object_server()
            .interface::<_, Settings>("/org/freedesktop/portal/desktop").await?;

        let mut iface = iface_ref.get_mut().await;
        
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
                    iface.change_setting(&conn, "org.freedesktop.appearance", "color-scheme", U32(2)).await;
                    
                }
                
            // Enable dark mode before sunrise/after sunset
            } else if !self.is_dark_mode {
                
                self.is_dark_mode = true;
                iface.change_setting(&conn, "org.freedesktop.appearance", "color-scheme", U32(1)).await;
                
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