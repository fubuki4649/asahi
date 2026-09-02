use crate::_utils::mutex_ext::MutexExt;
use crate::config::load_config;
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
mod hooks;


static CONTEXT: LazyLock<Mutex<Context>> = LazyLock::new(|| {
    Mutex::new(Context::new())
});

static PORTAL: LazyLock<Mutex<PortalConnection>> = LazyLock::new(|| {
    Mutex::new(PortalConnection::new().unwrap_or_else(|e| panic!("Failed to initialize D-Bus portal: {e}")))
});

fn main() {

    // Load log level from config before anything else, so all subsequent log calls respect it.
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
        ctx.on_cleanup();
        drop(ctx);

        // Broadcast dark mode = unset before exiting
        let portal = PORTAL.lock_recover();
        portal.broadcast_darkmode(0);
        drop(portal);

        // Exit with code 0
        std::process::exit(0);
    }).unwrap_or_else(|e| warn!("Failed to set exit hook: {e}"));


    // Broadcast immediately on startup so clients don't have to wait up to
    // `sunset_check_frequency` seconds for the first mode signal.
    broadcast_current_mode();

    loop {
        sleep(Duration::from_secs({
            let ctx = CONTEXT.lock_recover();
            ctx.sunset_check_frequency
        }));

        broadcast_current_mode();
    }

}

/// Calculates the current dark mode value and — if it has changed since the
/// last broadcast — emits a D-Bus signal and runs the appropriate hooks.
fn broadcast_current_mode() {
    let mut ctx = CONTEXT.lock_recover();
    if ctx.override_theme != -1 { return; }

    let new_value = ctx.calculate_dark_mode();
    drop(ctx);

    // If the color theme has changed from the previous broadcast, broadcast the new value and run hooks
    let mut portal = PORTAL.lock_recover();
    if portal.prev_broadcast_val != new_value {
        portal.prev_broadcast_val = new_value;
        
        portal.broadcast_darkmode(new_value);
        drop(portal);
        
        hooks::run_hooks(new_value);
    }
}
