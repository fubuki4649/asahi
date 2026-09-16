use crate::_utils::mutex_ext::MutexExt;
use crate::config::{load_config, Value};
use crate::context::Context;
use crate::dbus::listener::{spawn_listeners, WakeEvent};
use crate::dbus::portals::portal_connection::PortalConnection;
use log::warn;
use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{mpsc, LazyLock, Mutex};

mod dbus;
pub mod context;
mod location;
mod _utils;
mod config;
mod hooks;
pub mod sun;

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
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<log::Level>().ok())
        .unwrap_or(log::Level::Info);

    simple_logger::init_with_level(log_level).unwrap();

    // Set exit hook
    match Signals::new([SIGINT, SIGTERM, SIGHUP]) {
        Ok(mut signals) => {
            std::thread::spawn(move || {
                // Wait for SIGINT, SIGTERM, or SIGHUP on a different thread
                if signals.forever().next().is_some() {
                    // Persist the last known location to the cache before exiting
                    let ctx = CONTEXT.lock_recover();
                    ctx.on_cleanup();
                    drop(ctx);

                    // Broadcast dark mode = unset before exiting
                    let mut portal = PORTAL.lock_recover();
                    portal.broadcast_darkmode(0);
                    drop(portal);

                    // Exit with code 0
                    std::process::exit(0);
                }
            });
        }
        Err(e) => warn!("Failed to set exit hook: {e}"),
    }


    // Broadcast immediately on startup
    broadcast_current_theme(false);
    broadcast_current_theme(true);

    let (tx, rx) = mpsc::channel::<WakeEvent>();
    spawn_listeners(tx);

    loop {
        let sleep_period = CONTEXT.lock_recover().next_wakeup();

        match rx.recv_timeout(sleep_period) {
            Ok(WakeEvent::SystemResume) => {
                // Offline fast-path: use cached location, correct theme immediately on wake
                // This is primarily to catch situations where the device has slept past a solar event
                broadcast_current_theme(false);
            }
            Ok(WakeEvent::NetworkConnect) => {
                // Just connected to a network: refresh location and update themes
                broadcast_current_theme(true);
            }
            Err(RecvTimeoutError::Timeout) => {
                // Standard wakeup - default behavior (refresh location and timing)
                broadcast_current_theme(false);
                broadcast_current_theme(true);
            }
            Err(_) => {
                log::error!("All listener threads have died unexpectedly. This should basically never happen and is a likely a d-bus problem.");
                panic!("Catastrophic failure!");
            }
        }
    }
}


/// Calculates the current dark mode value and — if it has changed since the
/// last broadcast — emits a D-Bus signal and runs the appropriate hooks.
///
/// `with_location` - Also updates the location and forces a recalculation of today's sunrise/sunset
/// times before broadcasting
pub fn broadcast_current_theme(with_location: bool) {
    let mut ctx = CONTEXT.lock_recover();
    // Do nothing if an override is set
    if ctx.sun_stats.has_override() { return; }

    // Refresh location before getting dark mode if `with_location` is set.
    if with_location { ctx.update_location() }
    let new_theme_value = ctx.calculate_dark_mode(with_location);
    drop(ctx);

    // If the color theme has changed from the previous broadcast, broadcast the new value and run hooks
    let mut portal = PORTAL.lock_recover();
    if portal.prev_broadcast_val != new_theme_value {
        portal.broadcast_darkmode(new_theme_value);
        drop(portal);
        
        hooks::run_hooks(new_theme_value);
    }
}
