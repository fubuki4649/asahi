use crate::dbus_portal::portal::Portal;
use anyhow::Error;
use zbus::block_on;
use zbus::blocking::{connection, Connection};
use zbus::zvariant::Value::U32;
use crate::dbus_portal::control::Control;

pub struct PortalWrapper {
    conn: Connection,
}

impl PortalWrapper {
    pub fn new() -> Result<Self, Error> {
        // Initialize a DBus connection
        let conn = connection::Builder::session()?
                .name("org.freedesktop.impl.portal.desktop.asahi")?
                .serve_at("/org/freedesktop/portal/desktop", Portal::new())?
                .serve_at("/org/freedesktop/portal/desktop", Control::new())?
                .build()?;

        Ok(Self { conn })
    }

    /// 0 - No Preference
    ///
    /// 1 - Dark Mode
    ///
    /// 2 - Light Mode
    pub fn set_darkmode(&self, value: u32) {
        let iref = self.conn.object_server()
            .interface::<_, Portal>("/org/freedesktop/portal/desktop")
            .expect("Interface not found at path");

        block_on(iref.get_mut().change_setting(iref.signal_emitter(), "org.freedesktop.appearance", "color-scheme", U32(value)));
    }
}