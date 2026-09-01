use std::sync::{Mutex, MutexGuard};

/// Extension trait for [`Mutex`] that recovers from lock poisoning instead of panicking.
///
/// If a thread panics while holding one of the shared locks (e.g. `CONTEXT` or `PORTAL`),
/// the mutex becomes "poisoned" and a plain `.lock().unwrap()` would panic on every
/// subsequent access, cascading a single bug into a total daemon crash (including the
/// exit hook, which would then fail to reset the dark mode setting). Since this is a
/// long-running background daemon, it is preferable to recover the (possibly
/// inconsistent) inner state and keep running rather than panic indefinitely.
pub trait MutexExt<T> {
    fn lock_recover(&self) -> MutexGuard<'_, T>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn lock_recover(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
