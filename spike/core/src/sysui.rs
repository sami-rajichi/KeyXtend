//! Windows' look for the spike face, read through the product's `WinLook`; the mode and high-contrast maths moved
//! to `kx-theme` (P3), which reads the same `SysLook`.

use kx_platform::ui::{SysLook, SystemLook as _};
use kx_platform_windows::WinLook;

pub use kx_platform::ui::WatchGuard;

/// Reads Windows' look now.
pub fn read() -> SysLook {
    WinLook.read()
}

/// Calls `on_change` on the product's watcher thread with each new look, until the guard drops.
pub fn watch(on_change: impl Fn(SysLook) + Send + 'static) -> Result<WatchGuard, String> {
    WinLook
        .watch(Box::new(on_change))
        .map_err(|e| e.to_string())
}
