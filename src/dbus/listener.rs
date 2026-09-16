use log::{info, warn};
use std::sync::mpsc::Sender;
use zbus::blocking::{Connection, Proxy};

/// Events that can interrupt the main sleep loop early.
pub enum WakeEvent {
    SystemResume,
    NetworkConnect,
}

/// Spawns background threads that listen for system events and send [`WakeEvent`]s
/// to the main loop, interrupting its sleep early when relevant events occur.
pub fn spawn_listeners(tx: Sender<WakeEvent>) {
    spawn_listener("sleep-listener", tx.clone(), run_sleep_listener);
    spawn_listener("network-listener", tx, run_network_listener);
}

/// Spawns a new thread with a listener; if fails, logs the error and fails silently
fn spawn_listener(name: &'static str, tx: Sender<WakeEvent>, run: fn(&Sender<WakeEvent>) -> Result<(), zbus::Error>) {
    if let Err(e) = std::thread::Builder::new()
        .name(name.into())
        .spawn(move || {
            run(&tx).unwrap_or_else(|e| warn!("Failed to run {name}: {e}"));
        })
    {
        warn!("Failed to spawn {name}: {e}");
    }
}

/// Listens for `org.freedesktop.login1.Manager.PrepareForSleep(false)`,
/// which fires the moment the system resumes from sleep or hibernate.
fn run_sleep_listener(tx: &Sender<WakeEvent>) -> Result<(), zbus::Error> {
    let conn = Connection::system()?;
    let proxy = Proxy::new(
        &conn,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )?;

    for signal in proxy.receive_signal("PrepareForSleep")? {
        // Signal body is a bool: true = entering sleep, false = resuming
        if signal.body().deserialize::<bool>().unwrap_or(true) { continue; }
        info!("Resume from sleep detected");
        let _ = tx.send(WakeEvent::SystemResume);
    }

    Ok(())
}

/// Listens for `org.freedesktop.NetworkManager.StateChanged(u)`.
/// Sends a [`WakeEvent::NetworkConnect`] when NM reaches global connectivity (state 70).
fn run_network_listener(tx: &Sender<WakeEvent>) -> Result<(), zbus::Error> {
    let conn = Connection::system()?;
    let proxy = Proxy::new(
        &conn,
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager",
        "org.freedesktop.NetworkManager",
    )?;

    for signal in proxy.receive_signal("StateChanged")? {
        // NM_STATE_CONNECTED_GLOBAL = 70
        if signal.body().deserialize::<u32>().unwrap_or(0) != 70 { continue; }
        info!("Network connection event detected");
        let _ = tx.send(WakeEvent::NetworkConnect);
    }

    Ok(())
}
