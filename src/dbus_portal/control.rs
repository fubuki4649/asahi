use zbus::interface;
use crate::{CONTEXT, PORTAL};

pub struct Control {

}

impl Control {
    pub fn new() -> Self {
        Self {}
    }
}

#[interface(name = "org.freedesktop.impl.portal.asahi.Control")]
impl Control {

    /// ManualCtl method - used by the CLI tool to manually set dark mode
    /// -1 = Automatic
    /// 0 = No Preference
    /// 1 = Dark Mode
    /// 2 = Light Mode
    fn set_manual_darkmode(&self, code: i32) {
        // Store dark mode setting
        let mut ctx = CONTEXT.lock().unwrap();
        ctx.manual_darkmode = code;

        // Broadcast signal
        let conn = PORTAL.lock().unwrap();
        conn.broadcast_darkmode(ctx.calculate_dark_mode());

        drop(conn);
        drop(ctx);
    }

    /// Allow querying of current manual control setting as a property
    #[zbus(property, name = "manualDarkmodeSetting")]
    fn current_darkmode_setting(&self) -> i32 {
        let ctx = CONTEXT.lock().unwrap();
        let current_setting = ctx.manual_darkmode;
        drop(ctx);
        current_setting
    }

}