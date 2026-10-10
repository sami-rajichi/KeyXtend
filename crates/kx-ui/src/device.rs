//! The shared graphics device's rules, apart from Qt (ADR-0021): how it is made when the pinned adapter is gone at
//! start, what happens when none can be made, and how a lost one is made again.

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

/// What making a lost device again needs; the app's goes through Qt, the tests' is a fake.
pub trait Remake {
    /// The device made.
    type Device;
    /// Pins the lowest-power adapter, found anew, as a driver update can change its place.
    fn repin(&mut self);
    /// A device on the pinned adapter; `None` when none can be made now.
    fn make(&mut self) -> Option<Self::Device>;
    /// Waits before the next try.
    fn pause(&mut self);
}

/// The rebuilds of one run, so a device lost again and again cannot spin.
#[derive(Debug, Default)]
pub struct Rebuilds(u32);

impl Rebuilds {
    /// A new shared device for a lost one's windows: each try pins the adapter anew, never drops the pin, and counts
    /// against `most`, with a pause after a failed one. `None` once the tries run out, or at once when a frame ended
    /// off the GUI thread (`gui_only` false), so each window makes its own.
    pub fn remake<R: Remake>(
        &mut self,
        most: u32,
        gui_only: bool,
        how: &mut R,
    ) -> Option<R::Device> {
        if !gui_only {
            tracing::warn!("a frame ended off the GUI thread, so each window makes its own device");
            return None;
        }
        let mut failed = false;
        while self.0 < most {
            self.0 += 1;
            if failed {
                how.pause();
            }
            how.repin();
            if let Some(device) = how.make() {
                tracing::warn!(
                    rebuild = self.0,
                    "the graphics device was lost, so the windows are made again"
                );
                return Some(device);
            }
            failed = true;
            tracing::warn!(
                rebuild = self.0,
                "no graphics device could be made after the loss"
            );
        }
        tracing::warn!(
            most,
            "the graphics device was lost too often, so each window makes its own"
        );
        None
    }
}
