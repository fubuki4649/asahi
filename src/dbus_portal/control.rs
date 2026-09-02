use crate::_utils::mutex_ext::MutexExt;
use crate::{hooks, CONTEXT, PORTAL};
use zbus::interface;

pub struct Control;

impl Control {
    pub fn new() -> Self {
        Self {}
    }
}

#[interface(name = "org.freedesktop.impl.portal.asahi.Control")]
impl Control {

    /// `ManualCtl` method - used by the CLI tool to manually set dark mode
    /// -1 = Automatic
    /// 0 = No Preference
    /// 1 = Dark Mode
    /// 2 = Light Mode
    #[zbus(name = "setManualDarkMode")]
    #[allow(clippy::unused_self)]
    fn set_manual_darkmode(&self, override_mode: i32) {
        let mut ctx = CONTEXT.lock_recover();
        
        // Set override mode and get the new dark mode value.
        ctx.override_theme = override_mode;
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

    /// Allow querying of current manual control setting as a property
    #[zbus(property, name = "manualDarkModeSetting")]
    #[allow(clippy::unused_self)]
    fn current_darkmode_setting(&self) -> i32 {
        let ctx = CONTEXT.lock_recover();
        let current_setting = ctx.override_theme;
        drop(ctx);
        current_setting
    }

}