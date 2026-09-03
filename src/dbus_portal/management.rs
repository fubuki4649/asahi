use crate::_utils::mutex_ext::MutexExt;
use crate::{hooks, CONTEXT, PORTAL};
use chrono::Local;
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

    /// Allow querying of the current manual control setting as a property
    #[zbus(property, name = "isOverrideSet")]
    #[allow(clippy::unused_self)]
    fn is_override_set(&self) -> bool {
        let ctx = CONTEXT.lock_recover();
        let has_override = ctx.override_theme != -1;
        drop(ctx);
        has_override
    }

    /// Allow querying of the dark mode value currently being broadcast over D-Bus
    /// (0 = No Preference, 1 = Dark Mode, 2 = Light Mode)
    #[zbus(property, name = "currentTheme")]
    #[allow(clippy::unused_self)]
    fn current_theme(&self) -> u32 {
        let portal = PORTAL.lock_recover();
        let theme = portal.prev_broadcast_val;
        drop(portal);
        theme
    }

    /// Allow querying of the next expected sunrise/sunset transition, as an
    /// RFC 3339 timestamp in the local timezone.
    #[zbus(property, name = "nextTransitionAt")]
    #[allow(clippy::unused_self)]
    fn next_transition_at(&self) -> String {
        let mut ctx = CONTEXT.lock_recover();
        let next_transition = ctx.next_transition_at();
        drop(ctx);
        next_transition.with_timezone(&Local).to_rfc3339()
    }

    /// Allow querying of the latitude/longitude currently used for sunrise/sunset
    /// calculations, useful for debugging incorrect location data.
    #[zbus(property, name = "location")]
    #[allow(clippy::unused_self)]
    fn location(&self) -> (f64, f64) {
        let ctx = CONTEXT.lock_recover();
        let location = ctx.location();
        drop(ctx);
        (location.lat, location.lon)
    }

}