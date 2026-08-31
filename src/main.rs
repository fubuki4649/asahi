use crate::_utils::mutex_ext::MutexExt;
use crate::config::SUNSET_CHECK_FREQUENCY;
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
    Mutex::new(PortalConnection::new().unwrap_or_else(|e| panic!("Failed to initialize D-Bus portal: {}", e)))
});

fn main() {

    simple_logger::init_with_level(log::Level::Debug).unwrap();

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
    }).unwrap_or_else(|e| warn!("Failed to set exit hook: {}", e));


    loop {
        {
            let mut ctx = CONTEXT.lock_recover();

            // Check for sunset/sunrise if manual darkmode isn't set
            if ctx.manual_darkmode == -1 {
                let portal = PORTAL.lock_recover();
                portal.broadcast_darkmode(ctx.calculate_dark_mode());
                // portal is dropped here
            }
            // ctx is dropped here, before the sleep
        }

        sleep(Duration::from_secs(SUNSET_CHECK_FREQUENCY));
    }
    
}
