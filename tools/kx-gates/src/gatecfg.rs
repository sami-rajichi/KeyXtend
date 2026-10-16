//! Settings of the first gates: G1 typing, G2 and G4 clicks and G3 top band.
#![cfg(windows)]

use kx_mod_keyboard::KeyboardSettings;
use serde::Deserialize;

/// G1 typing settings.
#[derive(Debug, Clone, Deserialize)]
pub struct G1 {
    /// Characters typed per run.
    pub count: usize,
    /// Characters per `SendInput` call.
    pub chunk_chars: usize,
    /// Pause after each chunk, in ms.
    pub chunk_pause_ms: u64,
    /// Title of the Chrome test page, used to find its window.
    pub page_title: String,
    /// The Chrome test page's body: one text box that fills the page and takes the focus.
    pub page_body: String,
    /// Characters shown on each side of the first difference.
    pub context_chars: usize,
    /// Longest clipboard text we read, in characters; more is an error.
    pub max_read_chars: usize,
}

/// G2 click settings.
#[derive(Debug, Clone, Deserialize)]
pub struct G2 {
    /// Clicks per run.
    pub clicks: usize,
    /// Pause after each click before checking focus and caret, in ms.
    pub click_gap_ms: u64,
    /// A character belongs to a click if it arrives within this time, in ms.
    pub match_window_ms: i64,
    /// Wait for the last characters to reach the log, in ms.
    pub drain_ms: u64,
    /// Failures listed in the report.
    pub failures_shown: usize,
    /// Failures that stop the run.
    pub max_failures: usize,
    /// How long each click holds the button down, as the owner's hand does, in ms.
    pub press_ms: u64,
    /// How far each click slips right and down while the button is down, in physical px.
    pub slip_px: i32,
    /// How far below the keyboard's repeat delay each click's hold stays, in ms.
    pub repeat_margin_ms: u64,
}

impl G2 {
    /// Refuses a click with no hold or no slip, or a hold the default keyboard would repeat.
    pub fn check(&self) -> Result<(), String> {
        if self.press_ms == 0 || self.slip_px <= 0 {
            return Err("g2.press_ms and g2.slip_px must be above 0".to_string());
        }
        let defaults = KeyboardSettings::from_defaults()
            .map_err(|e| format!("the keyboard's defaults: {e}"))?;
        self.holds_below(&defaults)
    }

    /// Refuses a hold that, with its margin, reaches the repeat delay of a keyboard with `settings`, as each click
    /// would then repeat.
    pub fn holds_below(&self, settings: &KeyboardSettings) -> Result<(), String> {
        below_delay(
            "g2.press_ms",
            self.press_ms,
            self.repeat_margin_ms,
            settings,
        )
    }
}

/// Refuses setting `name`, a hold of `hold_ms`, that with `margin_ms` reaches the repeat delay of a keyboard with
/// `settings`, as each press would then repeat.
pub fn below_delay(
    name: &str,
    hold_ms: u64,
    margin_ms: u64,
    settings: &KeyboardSettings,
) -> Result<(), String> {
    let (delay, held) = (settings.repeat_delay_ms, hold_ms.saturating_add(margin_ms));
    if settings.repeat && held >= delay {
        return Err(format!(
            "{name} plus g2.repeat_margin_ms is {held} ms, not below the keyboard's repeat delay of {delay} ms: \
             each press would repeat"
        ));
    }
    Ok(())
}

/// G3 top-band settings.
#[derive(Debug, Clone, Deserialize)]
pub struct G3 {
    /// Wait after opening a surface before probing, in ms.
    pub open_wait_ms: u64,
    /// Board ids of the probed keys, which UI Automation finds on the product keyboard.
    pub probe_keys: Vec<String>,
    /// System surfaces opened over the keyboard.
    pub surfaces: Vec<Surface>,
}

impl G3 {
    /// Refuses a run that probes nothing or a key the product's board does not have.
    pub fn check(&self) -> Result<(), String> {
        if self.probe_keys.is_empty() {
            return Err("g3.probe_keys must name at least one key".to_string());
        }
        let ids = crate::board::ids()?;
        match self.probe_keys.iter().find(|k| !ids.contains(k)) {
            Some(k) => Err(format!(
                "g3.probe_keys: {k} is no key of the product's board"
            )),
            None => Ok(()),
        }
    }
}

/// G4 admin-window settings; click pauses and failure limits come from G2.
#[derive(Debug, Clone, Deserialize)]
pub struct G4 {
    /// Clicks per run.
    pub clicks: usize,
}

/// A system surface to open over the keyboard.
#[derive(Debug, Clone, Deserialize)]
pub struct Surface {
    /// Name in the report.
    pub name: String,
    /// Virtual keys that open it.
    pub open: Vec<u16>,
    /// Virtual keys that close it.
    #[serde(default)]
    pub close_keys: Vec<u16>,
    /// Class of a new window to close with `WM_CLOSE`.
    #[serde(default)]
    pub close_class: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g2() -> G2 {
        crate::config::load().expect("kx-gates.toml loads").g2
    }

    #[test]
    fn g2_clicks_as_the_owner_does_held_a_while_with_a_slip() {
        let g = g2();
        assert_eq!(g.check(), Ok(()));
        assert!(g.press_ms > 0 && g.slip_px > 0);
    }

    #[test]
    fn g2_refuses_no_hold_a_hold_that_would_repeat_and_no_slip() {
        let defaults = KeyboardSettings::from_defaults().expect("the defaults load");
        let close = defaults.repeat_delay_ms - g2().repeat_margin_ms;
        for (press_ms, slip_px) in [(0, 2), (close, 2), (400, 0), (400, -2)] {
            let g = G2 {
                press_ms,
                slip_px,
                ..g2()
            };
            assert!(g.check().is_err(), "{press_ms} ms, {slip_px} px");
        }
    }

    #[test]
    fn a_hold_stays_a_margin_below_the_running_keyboards_repeat_delay() {
        let g = g2();
        let mut running = KeyboardSettings::from_defaults().expect("the defaults load");
        assert_eq!(g.holds_below(&running), Ok(()));
        running.repeat_delay_ms = g.press_ms + g.repeat_margin_ms;
        let e = g.holds_below(&running).expect_err("a click would repeat");
        assert!(e.contains("repeat"), "{e}");
        running.repeat = false;
        assert_eq!(
            g.holds_below(&running),
            Ok(()),
            "a keyboard that never repeats"
        );
    }

    #[test]
    fn a_margin_past_any_delay_is_refused_without_a_panic() {
        let g = G2 {
            repeat_margin_ms: u64::MAX,
            ..g2()
        };
        let running = KeyboardSettings::from_defaults().expect("the defaults load");
        assert!(g.holds_below(&running).is_err());
    }
}
