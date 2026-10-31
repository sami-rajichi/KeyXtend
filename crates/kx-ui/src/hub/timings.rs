//! The timings, paging, press frame and frame log QML reads, as one JSON.

use super::{Hub, json};
use crate::settings::PressFrame;
use serde::Serialize;

/// The timings, paging, press frame and frame log QML reads, as JSON.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QmlSettings {
    tip_ms: u64,
    poll_ms: u64,
    guard_delay_ms: u64,
    tray_grace_ms: u64,
    page_rows: usize,
    press_frame: PressFrame,
    press_frame_ms: u64,
    frame_log: bool,
}

impl Hub {
    /// The timings, paging, press frame and frame log QML reads, as JSON; the language comes apart, in `lang_json`.
    #[must_use]
    pub fn settings_json(&self) -> String {
        let s = self.settings();
        json(&QmlSettings {
            tip_ms: s.tip_ms,
            poll_ms: s.poll_ms,
            guard_delay_ms: s.guard_delay_ms,
            tray_grace_ms: s.tray_grace_ms,
            page_rows: s.page_rows,
            press_frame: s.press_frame,
            press_frame_ms: s.press_frame_ms,
            frame_log: s.frame_log,
        })
    }
}
