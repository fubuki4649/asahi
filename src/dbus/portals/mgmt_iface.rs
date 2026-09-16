use crate::_utils::mutex_ext::MutexExt;
use crate::sun::sun_stats::SunStats;
use crate::{broadcast_current_theme, hooks, CONTEXT, PORTAL};
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
    /// -1 = No Override
    /// 0 = No Preference
    /// 1 = Dark Mode
    /// 2 = Light Mode
    #[zbus(name = "setManualDarkMode")]
    #[allow(clippy::unused_self)]
    fn set_manual_darkmode(&self, override_mode: i32) {
        let mut ctx = CONTEXT.lock_recover();
        
        // Set override mode and get the new dark mode value.
        ctx.set_theme_override(override_mode);
        let new_value = ctx.calculate_dark_mode(true);
        
        drop(ctx);

        // If the color theme has changed from the previous broadcast, broadcast the new value and run hooks
        let mut portal = PORTAL.lock_recover();
        if portal.prev_broadcast_val != new_value {
            portal.broadcast_darkmode(new_value);
            drop(portal);

            hooks::run_hooks(new_value);
        }
        
    }

    /// `forceUpdate` method - used by the CLI tool to force a location refresh and theme rebroadcast.
    /// Equivalent to an immediate wake-up cycle: updates location, recalculates sunrise/sunset,
    /// and emits a D-Bus signal + runs hooks if the theme value changed.
    /// No-op when a manual override is active.
    #[zbus(name = "forceUpdate")]
    #[allow(clippy::unused_self)]
    fn force_update(&self) {
        broadcast_current_theme(true);
    }

    /// Allow querying of the current manual control setting as a property
    #[zbus(property, name = "isOverrideSet")]
    #[allow(clippy::unused_self)]
    fn is_override_set(&self) -> bool {
        let ctx = CONTEXT.lock_recover();
        let has_override = ctx.sun_stats.has_override();
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

    /// Expected time of today's sunrise/sunset as RFC 3339 timestamps in the local timezone.
    /// Returns empty strings when a manual override is active.
    #[zbus(property, name = "todayTransitionTimes")]
    #[allow(clippy::unused_self)]
    fn today_transition_times(&self) -> (String, String) {
        let mut ctx = CONTEXT.lock_recover();
        // Trigger the stale-data check so sun_stats reflects today's wall-clock date.
        ctx.calculate_dark_mode(false);
        let result = match &ctx.sun_stats {
            SunStats::Calculated(stats) => (
                stats.sunrise.naive_local().format("%Y-%m-%d %I:%M:%S %p").to_string(),
                stats.sunset.naive_local().format("%Y-%m-%d %I:%M:%S %p").to_string(),
            ),
            SunStats::Override(_) => Default::default(),
        };
        drop(ctx);
        result
    }

}