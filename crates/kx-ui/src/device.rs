//! The shared graphics device's rules, apart from Qt (ADR-0021): how it is made when the pinned adapter is gone, what
//! happens when none can be made, and how often a lost one is made again.

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

/// What the windows draw on once a lost device's windows are made again.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Rebuild {
    /// A new shared device.
    Shared,
    /// Each window's own device, as Qt makes them: the rebuilds ran out.
    Own,
}

/// The rebuilds of one run, so a device lost again and again cannot spin.
#[derive(Debug, Default)]
pub struct Rebuilds(u32);

impl Rebuilds {
    /// The next rebuild: on a new shared device while fewer than `most` were made, else on each window's own; each
    /// warns.
    pub fn next(&mut self, most: u32) -> Rebuild {
        if self.0 < most {
            self.0 += 1;
            tracing::warn!(
                rebuild = self.0,
                "the graphics device was lost, so the windows are made again"
            );
            Rebuild::Shared
        } else {
            tracing::warn!(
                most,
                "the graphics device was lost too often, so each window makes its own"
            );
            Rebuild::Own
        }
    }
}
