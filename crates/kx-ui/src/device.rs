//! The shared graphics device's rules, apart from Qt (ADR-0021): how it is made when the pinned adapter is gone, and
//! what happens when none can be made.

/// Makes the shared device with `make`; when that fails, `unpin` drops the adapter pin, true when there was one, and
/// `make` tries Qt's default adapter once. `None` means each window makes its own device, as Qt does alone.
pub fn share<T>(mut make: impl FnMut() -> Option<T>, unpin: impl FnOnce() -> bool) -> Option<T> {
    if let Some(device) = make() {
        return Some(device);
    }
    let retried = if unpin() { make() } else { None };
    if retried.is_some() {
        tracing::warn!(
            "the pinned graphics adapter is gone, so the device is made on Qt's default"
        );
    } else {
        tracing::warn!("no shared graphics device could be made, so each window makes its own");
    }
    retried
}
