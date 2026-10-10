//! The shared graphics device's rules, apart from Qt (ADR-0021): how it is made when the pinned adapter is gone at
//! start, what happens when none can be made, and how a lost one is made again, one try at a time.

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

/// What making a lost device again needs; the app's goes through Qt, the tests' is a fake. Nothing here waits: the next
/// try comes through Qt's loop.
pub trait Remake {
    /// The device made.
    type Device;
    /// Pins the lowest-power adapter, found anew, as a driver update can change its place.
    fn repin(&mut self);
    /// A device on the pinned adapter; `None` when none can be made now.
    fn make(&mut self) -> Option<Self::Device>;
}

/// Why the windows are made again.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cause {
    /// A frame found the shared device lost; `gui_only` is false when a frame ran off the GUI thread.
    Lost {
        /// Every frame on the device ended on the GUI thread.
        gui_only: bool,
    },
    /// A window could not start drawing, as when it cannot make its own device during a reset.
    Failed,
}

/// What the windows of a lost device do next.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Next<T> {
    /// They are made again on this new shared device.
    Shared(T),
    /// Each is made again on its own device, as Qt makes them.
    Own,
    /// No device could be made yet: the next try comes after `[ui] device_retry_ms`, with Qt's loop running.
    Later,
    /// No window can draw and the tries are used: the app ends with a failure.
    End,
}

/// The rebuilds of one run, so a device lost again and again cannot spin.
#[derive(Debug, Default)]
pub struct Rebuilds(u32);

impl Rebuilds {
    /// One try at a new shared device for `cause`: it pins the adapter anew, never drops the pin, and counts against
    /// `most`. A frame off the GUI thread gives each window its own device at once.
    pub fn next<R: Remake>(&mut self, most: u32, cause: Cause, how: &mut R) -> Next<R::Device> {
        if cause == (Cause::Lost { gui_only: false }) {
            tracing::warn!("a frame ended off the GUI thread, so each window makes its own device");
            return Next::Own;
        }
        if self.0 >= most {
            return spent(cause);
        }
        self.0 += 1;
        how.repin();
        if let Some(device) = how.make() {
            tracing::warn!(
                rebuild = self.0,
                "the windows are made again on a new graphics device"
            );
            return Next::Shared(device);
        }
        tracing::warn!(
            rebuild = self.0,
            "no graphics device could be made after the loss"
        );
        if self.0 < most {
            Next::Later
        } else {
            spent(cause)
        }
    }
}

/// What follows once the tries are used: each window's own device after a loss, or the end when a window could not
/// draw even on its own.
fn spent<T>(cause: Cause) -> Next<T> {
    if cause == Cause::Failed {
        tracing::error!(
            "a window cannot draw and no new graphics device could be made, so the app ends"
        );
        return Next::End;
    }
    tracing::warn!("no new graphics device could be made, so each window makes its own");
    Next::Own
}
