use crate::dbus_portal::portal::Portal;
use anyhow::Error;
use log::info;
use zbus::blocking::{connection, Connection};
use zbus::zvariant::Value::U32;

pub struct PortalWrapper {
    conn: Connection,
}

impl PortalWrapper {
    pub fn new() -> Result<Self, Error> {
        // Initialize a DBus connection
        let conn = connection::Builder::session()?
                .name("org.freedesktop.impl.dbus_portal.desktop.asahi")?
                .serve_at("/org/freedesktop/dbus_portal/desktop", Portal::new())?
                .build()?;

        // Set dark mode to no preference before exiting
        ctrlc::set_handler(move || {
            info!("Portal: Exit Signal Received");
        })?;

        Ok(Self { conn })
    }

    /// 0 - No Preference
    ///
    /// 1 - Dark Mode
    ///
    /// 2 - Light Mode
    pub fn set_darkmode(&self, value: u32) {
        let iref = self.conn.object_server()
            .interface::<_, Portal>("/org/freedesktop/dbus_portal/desktop")
            .expect("Interface not found at path");

        let _ = iref.get_mut().change_setting(iref.signal_emitter(), "org.freedesktop.appearance", "color-scheme", U32(value));
    }
}