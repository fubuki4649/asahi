use crate::config::SUNSET_CHECK_FREQUENCY;
use crate::context::Context;
use crate::dbus_portal::portal_connection::PortalConnection;
use log::warn;
use std::sync::{LazyLock, Mutex};
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

static PORTAL: LazyLock<Mutex<PortalConnection>> = LazyLock::new(|| {
    Mutex::new(PortalConnection::new().unwrap_or_else(|e| panic!("Failed to initialize D-Bus portal: {}", e)))
});

fn main() {

    simple_logger::init_with_level(log::Level::Debug).unwrap();

    // Set exit hook
    ctrlc::set_handler(move || {
        // Broadcast dark mode = unset before exiting
        let portal = PORTAL.lock().unwrap();
        portal.broadcast_darkmode(0);
        drop(portal);

        // Exit with code 0
        std::process::exit(0);
    }).unwrap_or_else(|e| warn!("Failed to set exit hook: {}", e));


    loop {
        let mut ctx = CONTEXT.lock().unwrap();

        // Check for sunset/sunrise if manual darkmode isn't set
        if ctx.manual_darkmode == -1 {
            let portal = PORTAL.lock().unwrap();
            portal.broadcast_darkmode(ctx.calculate_dark_mode());

            drop(portal);
        }

        drop(ctx);
        sleep(Duration::from_secs(SUNSET_CHECK_FREQUENCY));
    }
    
}
