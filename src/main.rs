use std::sync::{LazyLock, Mutex};
use crate::config::SUNSET_CHECK_FREQUENCY;
use crate::context::Context;
use crate::dbus_portal::wrapper::PortalWrapper;
use chrono::Utc;
use log::{error, info, warn};
use std::thread::sleep;
use std::time::Duration;

mod dbus_portal;
mod context;
mod location;
mod _utils;
mod config;


static CONTEXT: LazyLock<Mutex<Context>> = LazyLock::new(|| {
    Mutex::new(Context::new())
});


fn main() {

    simple_logger::init_with_level(log::Level::Debug).unwrap();

    // Initiate portal
    let portal = match PortalWrapper::new() {
        Ok(portal) => portal,
        Err(e) => {
            error!("Failed to initialize D-Bus portal: {}", e);
            return;
        },
    };

    // Set exit hook
    ctrlc::set_handler(move || {
        std::process::exit(0);
    }).unwrap_or_else(|e| warn!("Failed to set exit hook: {}", e));


    loop {
        let mut ctx = CONTEXT.lock().unwrap();
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

        drop(ctx);
        sleep(Duration::from_secs(SUNSET_CHECK_FREQUENCY));
    }
    
}
