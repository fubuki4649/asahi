use crate::dbus_portal::portal::Portal;
use anyhow::Error;
use zbus::blocking::{connection, Connection};
use zbus::zvariant::Value::U32;

struct PortalInterface {
    conn: Connection,
}

impl PortalInterface {
    fn new() -> Result<Self, Error> {
        // Initialize a DBus connection
        let conn = connection::Builder::session()?
                .name("org.freedesktop.impl.dbus_portal.desktop.asahi")?
                .serve_at("/org/freedesktop/dbus_portal/desktop", Portal::new())?
                .build()?;

        // Set dark mode to no preference before exiting
        ctrlc::set_handler(move || {
            println!("Exit Signal Received");
        })?;

        Ok(Self { conn })
    }

    fn set_darkmode(&self, value: u32) {
        let iref = self.conn.object_server()
            .interface::<_, Portal>("/org/freedesktop/dbus_portal/desktop")
            .expect("Interface not found at path");

        let _ = iref.get_mut().change_setting(iref.signal_emitter(), "org.freedesktop.appearance", "color-scheme", U32(value));
    }
}