//! Physical pixels for one freeze: the thread reads every box, screen and pointer unscaled while held, whatever DPI
//! awareness the process's manifest gave it, so a part across screens of different scaling stays exact (edge 2).

use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetThreadDpiAwarenessContext,
};

/// This thread reads in another DPI awareness until dropped, then reads as it did before.
pub(crate) struct Aware(DPI_AWARENESS_CONTEXT);

impl Aware {
    /// Switches this thread to physical pixels; `None` when this Windows cannot.
    pub(crate) fn physical() -> Option<Self> {
        Self::to(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)
    }

    /// Switches this thread to `context`; `None` when Windows refuses it, the thread left as it was.
    #[allow(unsafe_code, reason = "Sets this thread's DPI awareness.")]
    pub(crate) fn to(context: DPI_AWARENESS_CONTEXT) -> Option<Self> {
        // SAFETY: a switch for this thread alone; Windows checks `context` and answers null, changing nothing, for a
        // value that is not one.
        let old = unsafe { SetThreadDpiAwarenessContext(context) };
        (!old.is_invalid()).then_some(Self(old))
    }
}

impl Drop for Aware {
    #[allow(unsafe_code, reason = "Puts this thread's DPI awareness back.")]
    fn drop(&mut self) {
        // SAFETY: the value this thread had before the switch, which Windows took then.
        let _ = unsafe { SetThreadDpiAwarenessContext(self.0) };
    }
}
