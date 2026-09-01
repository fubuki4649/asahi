use crate::_utils::mutex_ext::MutexExt;
use crate::config::load_config;
use crate::context::Context;
use crate::dbus_portal::portal_connection::PortalConnection;
use location::providers::provider_trait::LocationProvider;
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
    Mutex::new(PortalConnection::new().unwrap_or_else(|e| panic!("Failed to initialize D-Bus portal: {e}")))
});

fn main() {

    // Load log level from config before anything else so all subsequent log calls respect it.
    // Accepts: "error", "warn", "info", "debug", "trace". Defaults to "info".
    let log_level = load_config()
        .get("log_level")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<log::Level>().ok())
        .unwrap_or(log::Level::Info);

    simple_logger::init_with_level(log_level).unwrap();

    // Set exit hook
    ctrlc::set_handler(move || {
        // Persist the last known location to the cache before exiting
        let ctx = CONTEXT.lock_recover();
        ctx.location_provider.on_cleanup();
        drop(ctx);

        // Broadcast dark mode = unset before exiting
        let portal = PORTAL.lock_recover();
        portal.broadcast_darkmode(0);
        drop(portal);

        // Exit with code 0
        std::process::exit(0);
    }).unwrap_or_else(|e| warn!("Failed to set exit hook: {e}"));


    loop {
        // Snapshot the frequency before releasing the lock so the sleep
        // happens outside the critical section (as before).
        let sleep_duration = {
            let mut ctx = CONTEXT.lock_recover();

            // Check for sunset/sunrise if manual darkmode isn't set
            if ctx.manual_darkmode == -1 {
                let portal = PORTAL.lock_recover();
                portal.broadcast_darkmode(ctx.calculate_dark_mode());
                // portal is dropped here
            }

            ctx.sunset_check_frequency
            // ctx is dropped here, before the sleep
        };

        sleep(Duration::from_secs(sleep_duration));
    }
    
}
