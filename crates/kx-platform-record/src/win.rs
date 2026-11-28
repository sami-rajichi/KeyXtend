//! The recorder's Windows parts (ADR-0056 decisions 4 and 5): Windows Graphics Capture with each frame kept on the
//! graphics card, the microphone and the PC's sound through `cpal`, and Media Foundation's Sink Writer and Source
//! Reader, whose libraries load only when a video job asks for them.

mod capture;
mod com;
mod cut;
mod ears;
mod encode;
mod kinds;
mod media;
mod pace;
mod reel;
mod room;
mod studio;
mod writer;

pub(crate) use com::Com;
pub use studio::WinStudio;

use windows::Win32::Foundation::{ERROR_DISK_FULL, ERROR_HANDLE_DISK_FULL, WIN32_ERROR};
use windows::Win32::Media::MediaFoundation::MF_E_TOPO_CODEC_NOT_FOUND;
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetThreadDpiAwarenessContext,
};

use crate::record::RecordFault;

/// The fault a refused step stands for: `Full` when the drive filled up, `NoVideo` without an encoder, else `Failed`.
fn fault(e: &windows::core::Error) -> RecordFault {
    let full = [ERROR_DISK_FULL, ERROR_HANDLE_DISK_FULL].map(WIN32_ERROR::to_hresult);
    match e.code() {
        code if full.contains(&code) => RecordFault::Full,
        MF_E_TOPO_CODEC_NOT_FOUND => RecordFault::NoVideo,
        _ => RecordFault::Failed,
    }
}

/// `fault` as a function for `map_err`.
#[allow(
    clippy::needless_pass_by_value,
    reason = "`map_err` hands the error over."
)]
fn refused(e: windows::core::Error) -> RecordFault {
    fault(&e)
}

/// Makes this thread read the screen in physical pixels, as the app gives a part, and show windows sharp on every
/// monitor; false when the system refused. Only the video and dialog jobs' threads call it.
#[allow(unsafe_code, reason = "Sets this thread's DPI awareness.")]
pub(crate) fn physical() -> bool {
    // SAFETY: a plain switch for this thread alone; the old value it hands back is only checked.
    let old = unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
    !old.is_invalid()
}

/// The size of `T` as Windows takes it; every struct passed is far below the cap.
fn size32<T>() -> u32 {
    u32::try_from(size_of::<T>()).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
