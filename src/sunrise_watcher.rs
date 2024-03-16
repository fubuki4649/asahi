use std::error::Error;
use std::sync::Mutex;

use chrono::{Datelike, Utc};
use tokio::time::{Duration, sleep};
use zbus::{connection, Connection};
use zbus::zvariant::Value::U32;

use crate::asahi_state::AsahiState;
use crate::portal::Settings;

pub async fn observe_sunrise(state: &Mutex<AsahiState>) -> Result<(), Box<dyn Error>> {

    let conn = connection::Builder::session()?
        .name("org.freedesktop.impl.portal.desktop.asahi")?
        .serve_at("/org/freedesktop/portal/desktop", Settings::new())?
        .build().await?;

    loop {
        
        let now = Utc::now();

        let mut state_lock = state.lock().unwrap();
        
        // Check Date and make sure that sunrise/sunset times are for the current day
        if state_lock.year != now.year() || state_lock.month != now.month() || state_lock.day != now.day() {

            state_lock.calculate_sunrise();
            
        }
        
        // Check Dark Mode
        // Disable dark mode between sunrise and sunset
        if state_lock.sunrise <= now.timestamp() && now.timestamp() < state_lock.sunset {
            
            if state_lock.is_dark_mode {
                
                println!("Dark Mode Disabled");

                state_lock.is_dark_mode = false;
                set_darkmode(&conn, 2).await?;
                
            }
            
        // Enable dark mode before sunrise/after sunset
        } else if !state_lock.is_dark_mode {

            println!("Dark Mode Enabled");

            state_lock.is_dark_mode = true;
            set_darkmode(&conn, 1).await?;
            
        }

        drop(state_lock);
        
        // Sleep - Only check every minute
        sleep(Duration::from_secs(60)).await;
        
    }
    
}


async fn set_darkmode(conn: &Connection, value: u32) -> Result<(), Box<dyn Error>> {
    
    let iface_ref = conn
        .object_server()
        .interface::<_, Settings>("/org/freedesktop/portal/desktop").await?;
    
    let mut iface = iface_ref.get_mut().await;

    iface.change_setting(conn, "org.freedesktop.appearance", "color-scheme", U32(value)).await;
    
    Ok(())
    
}