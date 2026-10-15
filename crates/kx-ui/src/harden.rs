//! The variables Qt reads at start. Another program of the same user could set them to load its own code into the
//! uiAccess process, so the app clears them before Qt starts (see `harden_qt`).

use crate::settings::UiSettings;

/// The name prefixes of the variables Qt, Qt Quick and the QML engine read.
pub const QT_PREFIXES: [&str; 4] = ["QT_", "QML", "QSG_", "QV4_"];

/// The scene graph variables the app sets after the clear (ADR-0021): every window draws on the GUI thread, as one
/// shared device needs, and without a depth buffer.
pub const DRAW_VARS: [(&str, &str); 2] =
    [("QSG_RENDER_LOOP", "basic"), ("QSG_NO_DEPTH_BUFFER", "1")];

/// How many frames may wait to show, read as Qt's Direct3D device is made (`[ui] frame_latency`).
pub const LATENCY_VAR: &str = "QT_D3D_MAX_FRAME_LATENCY";
/// "1" turns off Qt's thread that starts frames on the screen's refresh (`[ui] vblank_updates`).
pub const NO_VBLANK_VAR: &str = "QT_D3D_NO_VBLANK_THREAD";
/// Qt's update timer in ms, where frames do not follow the refresh (`[ui] update_idle_ms`).
pub const IDLE_VAR: &str = "QT_QPA_UPDATE_IDLE_TIME";
/// The bytes of each buffer kind a window's renderer keeps to reuse, read as it is made (`[ui] buffer_pool_bytes`).
pub const POOL_VAR: &str = "QSG_RENDERER_BUFFER_POOL_LIMIT";
/// What Qt 6.11.2 keeps of each buffer kind per window when `POOL_VAR` is not set (`DEFAULT_BUFFER_POOL_SIZE_LIMIT`).
pub const QT_POOL_BYTES: u32 = 2 * 1024 * 1024;

/// Ends each name in a list of names, as it ends a C string.
pub const NAME_END: u8 = 0;

/// The frame timing and buffer pool variables the app sets after the clear, from the `[ui]` settings `s`. The timing
/// defaults are Qt's own, and the pool's keeps no spare buffer (ADR-0028).
#[must_use]
pub fn timing_vars(s: &UiSettings) -> [(&'static str, String); 4] {
    [
        (LATENCY_VAR, s.frame_latency.to_string()),
        (NO_VBLANK_VAR, u8::from(!s.vblank_updates).to_string()),
        (IDLE_VAR, s.update_idle_ms.to_string()),
        (POOL_VAR, s.buffer_pool_bytes.to_string()),
    ]
}

/// The swap interval every Qt Quick window asks for under `s`: 1 shows each frame at a screen refresh, 0 at once.
#[must_use]
pub fn swap_interval(s: &UiSettings) -> i32 {
    i32::from(s.present_vsync)
}

/// True for a variable Qt may read: a name that starts with a prefix, in any case, as Windows ignores case. The rest
/// may be any bytes, so every name with a Qt prefix goes, whatever the code page made of it.
#[must_use]
pub fn is_qt(name: &[u8]) -> bool {
    QT_PREFIXES.iter().any(|p| {
        name.get(..p.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(p.as_bytes()))
    })
}

/// The names in `list`, each ended by `NAME_END`, that Qt may read, in order.
#[must_use]
pub fn qt_names(list: &[u8]) -> Vec<&[u8]> {
    list.split(|b| *b == NAME_END)
        .filter(|n| is_qt(n))
        .collect()
}

#[cfg(test)]
mod tests;
