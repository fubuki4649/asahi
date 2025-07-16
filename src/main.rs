use crate::config::SUNSET_CHECK_FREQUENCY;
use crate::context::Context;
use crate::dbus_portal::wrapper::PortalWrapper;
use chrono::Utc;
use log::{error, info};
use std::thread::sleep;
use std::time::Duration;

mod dbus_portal;
mod context;
mod location;
mod _utils;
mod config;

fn main() {

    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let mut ctx = Context::new();

    let portal = match PortalWrapper::new() {
        Ok(portal) => portal,
        Err(e) => {
            error!("Failed to initialize D-Bus portal: {}", e);
            return;
        },
    };

    // TODO: Add ctrl-c handlers here


    loop {
        let now = Utc::now();

        // Update location/sunrise/sunset times
        ctx.update_location();
        ctx.update_sunrise();

        // Check for sunset/sunrise
        // Send light mode (2) signal if its daytime
        if ctx.sunrise <= now && now < ctx.sunset {
            portal.set_darkmode(2);
            info!("Set Light Mode!");
        }
        // Otherwise, set dark mode (1) signal
        else {
            portal.set_darkmode(1);
            info!("Set Dark Mode!");
        }

        sleep(Duration::from_secs(SUNSET_CHECK_FREQUENCY));
    }
    
}
