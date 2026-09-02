use std::ops::Deref;
use crate::dbus_portal::control::Control;
use crate::dbus_portal::xdg_interfaces::XDGInterfaces;
use anyhow::Error;
use log::info;
use zbus::block_on;
use zbus::blocking::{connection, Connection};
use zbus::zvariant::Value::U32;


pub struct PortalConnection {
    connection: Connection,
    /// The last mode value sent over D-Bus. Used by callers to suppress
    /// redundant broadcasts and decide when to fire transition hooks.
    pub prev_broadcast_val: u32,
}

impl Deref for PortalConnection {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}


impl PortalConnection {

    pub fn new() -> Result<Self, Error> {
        // Initialize a DBus connection
        let conn = connection::Builder::session()?
                .name("org.freedesktop.impl.portal.desktop.asahi")?
                .serve_at("/org/freedesktop/portal/desktop", XDGInterfaces::new())?
                .serve_at("/org/freedesktop/portal/desktop", Control::new())?
                .build()?;

        Ok(Self { connection: conn, prev_broadcast_val: 0 })
    }

    /// 0 - No Preference
    ///
    /// 1 - Dark Mode
    ///
    /// 2 - Light Mode
    pub fn broadcast_darkmode(&self, value: u32) {
        let iref = self.object_server()
            .interface::<_, XDGInterfaces>("/org/freedesktop/portal/desktop")
            .expect("Interface not found at path");

        block_on(iref.get_mut().change_setting(iref.signal_emitter(), "org.freedesktop.appearance", "color-scheme", U32(value)));
        info!("Set darkmode to {value}!");
    }
}