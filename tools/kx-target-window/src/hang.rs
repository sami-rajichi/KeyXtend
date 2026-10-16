//! The hang message: a gate asks the test window to sleep a while, as a hung app does, and the window caps that time.

/// The message's place after `WM_APP`, where Windows leaves numbers to an app's own messages.
pub const OFFSET: u32 = 1;

/// The hang message's number; its `wparam` holds the asked sleep in ms.
#[cfg(windows)]
pub const MESSAGE: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + OFFSET;

/// How long the window sleeps when asked for `asked_ms`: never past `max_ms`.
#[must_use]
pub fn capped(asked_ms: u64, max_ms: u64) -> u64 {
    asked_ms.min(max_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_asked_hang_is_capped_by_hang_max_ms() {
        assert_eq!(capped(500, 3_000), 500);
        assert_eq!(capped(3_000, 3_000), 3_000);
        assert_eq!(capped(u64::MAX, 3_000), 3_000);
        assert_eq!(capped(500, 0), 0, "a zero cap never hangs");
    }
}
