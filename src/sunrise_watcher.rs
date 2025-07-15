use std::sync::{Arc, Mutex};
use anyhow::Error;
use chrono::{Datelike, Utc};
use tokio::runtime::Runtime;
use tokio::time::{Duration, sleep};
use zbus::{connection, Connection};
use zbus::zvariant::Value::U32;

use crate::asahi_state::AsahiState;

pub async fn observe_sunrise(state: &Mutex<AsahiState>) -> Result<(), Error> {

    let conn = Arc::new(connection::Builder::session()?
        .name("org.freedesktop.impl.portal.desktop.asahi")?
        .serve_at("/org/freedesktop/portal/desktop", Settings::new())?
        .build().await?);

    let shared_conn = Arc::clone(&conn);

    // Set dark mode to no preference before exiting
    ctrlc::set_handler(move || {
        println!("Exit Signal Received");
        let handler_conn = Arc::clone(&shared_conn);

        Runtime::new().unwrap().block_on(async move {
            set_darkmode(&handler_conn, 0).await.expect("");
        });

        std::process::exit(0);
    })?;

    loop {

        let mut state_lock = state.lock().unwrap();

        // Wait for location_old to be acquired before starting main loop
        if state_lock.sunrise == 0 && state_lock.sunset == 0 {

            println!("Waiting For Location");
            drop(state_lock);

            sleep(Duration::from_secs(1)).await;
            continue;

        }

        let now = Utc::now();

        // Check Date and make sure that sunrise/sunset times are for the current day
        if state_lock.year != now.year() || state_lock.month != now.month() || state_lock.day != now.day() {

            state_lock.calculate_sunrise();

        }

        // Check Dark Mode
        // Disable dark mode between sunrise and sunset
        let mut new_val: u32  = u32::MAX;
        if state_lock.sunrise <= now.timestamp() && now.timestamp() < state_lock.sunset {

            if state_lock.is_dark_mode {
                new_val = 2
            }

            println!("Dark Mode Disabled");
            state_lock.is_dark_mode = true;

        // Enable dark mode before sunrise/after sunset
        } else {

            if !state_lock.is_dark_mode {
                new_val = 1;
            }

            println!("Dark Mode Enabled");
            state_lock.is_dark_mode = true;

        }

        drop(state_lock);
        if new_val != u32::MAX {
            set_darkmode(&conn, new_val).await?;
        }

        // Sleep - Only check every minute
        sleep(Duration::from_secs(60)).await;

    }

}


async fn set_darkmode(conn: &Connection, value: u32) -> Result<(), Error> {

    let iface_ref = conn
        .object_server()
        .interface::<_, Settings>("/org/freedesktop/portal/desktop").await?;

    let mut iface = iface_ref.get_mut().await;

    iface.change_setting(conn, "org.freedesktop.appearance", "color-scheme", U32(value)).await;

    Ok(())

}