use std::error::Error;

use chrono::{Datelike, Utc};
use tokio::time::{Duration, sleep};
use zbus::connection;
use zbus::zvariant::Value::U32;

use crate::asahi_state::AsahiState;
use crate::portal::Settings;

pub async fn observe_sunrise(state: & mut AsahiState) -> Result<(), Box<dyn Error>> {

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
            if state.year != now.year() || state.month != now.month() || state.day != now.day() {
                
                state.calculate_sunrise();
                
            }
            
            // Check Dark Mode
            // Disable dark mode between sunrise and sunset
            if state.sunrise <= now.timestamp() && now.timestamp() < state.sunset {
                
                if state.is_dark_mode {

                    state.is_dark_mode = false;
                    iface.change_setting(&conn, "org.freedesktop.appearance", "color-scheme", U32(2)).await;
                    
                }
                
            // Enable dark mode before sunrise/after sunset
            } else if !state.is_dark_mode {
                
                state.is_dark_mode = true;
                iface.change_setting(&conn, "org.freedesktop.appearance", "color-scheme", U32(1)).await;
                
            }
            
            // Sleep - Only check every minute
            sleep(Duration::from_secs(1)).await;
            
        }
        
    }
