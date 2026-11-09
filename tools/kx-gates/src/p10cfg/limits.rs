//! The key and wait checks of `[g19_product]`.
#![cfg(windows)]

use kx_mod_keyboard::KeyboardSettings;
use kx_mod_selection::SelSettings;
use kx_module_api::ports::{BASE_DPI, physical};
use kx_theme::Themes;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_NUMLOCK;

use crate::apps::{INPUT, same_name};
use crate::p5cfg::SEP;
use crate::p8cfg::wait;
use crate::p10cfg::{Bar, G19Product, caret_ms, selection_defaults};

/// The selection module's look at a box: its settle, then its read, in ms.
fn look_ms(s: &SelSettings) -> u64 {
    u64::from(s.settle_ms).saturating_add(u64::from(s.read_ms))
}

/// The theme's gap between the strip and its text at the keyboard's default size, in physical px at the base DPI.
pub fn default_gap() -> Result<i32, String> {
    let themes = Themes::shipped().map_err(|e| format!("theme {e}"))?;
    let keyboard =
        KeyboardSettings::from_defaults().map_err(|e| format!("the keyboard's defaults: {e}"))?;
    Ok(physical(themes.shape.chips.gap(keyboard.size), BASE_DPI))
}

/// True for a kx-ui id: a group, `SEP`, then a name.
fn id_like(id: &str) -> bool {
    id.split_once(SEP)
        .is_some_and(|(a, b)| !a.is_empty() && !b.is_empty())
}

impl G19Product {
    /// Refuses a ping that names a path or holds a quote or separator, a resize corner that is no kx-ui id, a text box
    /// or address-bar walk of no size, and a look of no length.
    pub(super) fn check_ping(&self) -> Result<(), String> {
        let file = |c: char| c.is_ascii_alphanumeric() || ".-_".contains(c);
        let exe = &self.ping_exe;
        if exe.is_empty() || !exe.chars().all(file) {
            return Err(
                "g19_product.ping_exe must name a plain program file: letters, digits, dots, dashes and underscores"
                    .into(),
            );
        }
        let word =
            |a: &String| !a.is_empty() && a.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
        if self.ping_args.is_empty() || !self.ping_args.iter().all(word) {
            return Err(
                "g19_product.ping_args must be plain switches, letters, digits and dashes".into(),
            );
        }
        if !id_like(&self.corner) {
            return Err(format!(
                "g19_product.corner must be a kx-ui id: a group, {SEP}, a name"
            ));
        }
        if self.bar_depth == 0 {
            return Err("g19_product.bar_depth must be 1 or more".into());
        }
        if self.box_size == 0 {
            return Err("g19_product.box_size must be 1 or more".into());
        }
        if self.look_s == 0 {
            return Err("g19_product.look_s must be 1 or more".into());
        }
        Ok(())
    }

    /// Refuses a Num Lock that is another key, no Shift+Home, Shift+End or caret key, a browser that is a path, and a
    /// console that would run no script of the gate's.
    pub(super) fn check_keys(&self) -> Result<(), String> {
        if self.numlock != [VK_NUMLOCK.0] {
            return Err("g19_product.numlock must be the Num Lock key alone".into());
        }
        if self.select_home.is_empty() {
            return Err("g19_product.select_home must hold a key".into());
        }
        if self.select_end.is_empty() {
            return Err("g19_product.select_end must hold a key".into());
        }
        if self.caret_left.is_empty() {
            return Err("g19_product.caret_left must hold a key".into());
        }
        if !id_like(&self.tray) {
            return Err(format!(
                "g19_product.tray must be a kx-ui id: a group, {SEP}, a name"
            ));
        }
        let plain = |b: &String| !b.is_empty() && !b.contains(['\\', '/', ':']);
        if self.browsers.is_empty() || !self.browsers.iter().all(plain) {
            return Err("g19_product.browsers must name plain program files".into());
        }
        let c = &self.console;
        if c.exe.is_empty() || c.class.is_empty() || !c.args.iter().any(|a| a.contains(INPUT)) {
            return Err(format!(
                "g19_product.console needs a program, a window class and {INPUT} in its arguments"
            ));
        }
        Ok(())
    }

    /// Refuses an empty address-bar table, a row for a browser not in `browsers` or named twice, and a row that
    /// matches by nothing or by an empty value.
    pub(super) fn check_bars(&self) -> Result<(), String> {
        let known = |b: &String| self.browsers.iter().any(|k| same_name(k, b));
        let all: Vec<&String> = self.bars.iter().flat_map(|r| &r.browsers).collect();
        let twice = |i: usize| all[..i].iter().any(|b| same_name(b, all[i]));
        let row_ok = |r: &Bar| {
            let by: Vec<&String> = r.ids.iter().chain(&r.classes).chain(&r.names).collect();
            !r.browsers.is_empty() && !by.is_empty() && by.iter().all(|v| !v.trim().is_empty())
        };
        let ok = self.bars.iter().all(row_ok) && all.iter().all(|b| known(b));
        if self.bars.is_empty() || !ok || (0..all.len()).any(twice) {
            return Err(
                "g19_product.bars: each row names browsers of g19_product.browsers, each once, and an id, class or \
                 name"
                    .into(),
            );
        }
        Ok(())
    }

    /// Refuses a wait outside 1 ms to `most_ms`, a negative slack, and a caret slack as long as the caret's Paste
    /// stays; then the hang's own rules, with `(front_ms, press_ms)` per window brought forward and per click.
    pub(super) fn check_waits(
        &self,
        most_ms: u64,
        (front_ms, press_ms): (u64, u64),
    ) -> Result<(), String> {
        let waits = [
            ("ready_ms", self.ready_ms),
            ("strip_wait_ms", self.strip_wait_ms),
            ("gone_wait_ms", self.gone_wait_ms),
            ("quiet_ms", self.quiet_ms),
            ("settle_ms", self.settle_ms),
            ("hang_ms", self.hang_ms),
            ("hang_lead_ms", self.hang_lead_ms),
            ("hung_quiet_ms", self.hung_quiet_ms),
            ("hung_wait_ms", self.hung_wait_ms),
            ("hang_margin_ms", self.hang_margin_ms),
            ("ping_ms", self.ping_ms),
            ("ping_exit_ms", self.ping_exit_ms),
            ("caret_slack_ms", self.caret_slack_ms),
            ("owner_wait_ms", self.owner_wait_ms),
        ];
        let key = |k: &str| format!("g19_product.{k}");
        waits
            .iter()
            .try_for_each(|(k, ms)| wait(&key(k), *ms, most_ms))?;
        self.check_module()?;
        self.check_place()?;
        self.check_hang(front_ms, press_ms)
    }

    /// Refuses a quiet wait the module's look outlasts, a wait for the strip to go as long as an edit's deadline, and
    /// a caret slack as long as the caret's Paste stays.
    fn check_module(&self) -> Result<(), String> {
        let s = selection_defaults()?;
        let look = look_ms(&s);
        if self.quiet_ms <= look {
            return Err(format!(
                "g19_product.quiet_ms must outlast the module's look, {look} ms"
            ));
        }
        let edit = u64::from(s.edit_ms);
        if self.gone_wait_ms >= edit {
            return Err(format!(
                "g19_product.gone_wait_ms must be shorter than the module's edit_ms, {edit} ms, so a strip an edit's \
                 deadline hid never counts as gone"
            ));
        }
        if self.caret_slack_ms >= caret_ms(&s) {
            return Err(
                "g19_product.caret_slack_ms must be shorter than the module's caret_s".into(),
            );
        }
        Ok(())
    }

    /// Refuses a place slack, or a near-bottom distance, that is not below the theme's gap at the keyboard's default
    /// size; the distance must be 1 px at least, so the strip has no room below the line.
    fn check_place(&self) -> Result<(), String> {
        let gap = default_gap()?;
        if !(0..gap).contains(&self.place_slack_px) {
            return Err(format!(
                "g19_product.place_slack_px must be 0 to {} px, below the theme's gap",
                gap.saturating_sub(1)
            ));
        }
        if !(1..gap).contains(&i32::from(self.near_px)) {
            return Err(format!(
                "g19_product.near_px must be 1 to {} px, below the theme's gap",
                gap.saturating_sub(1)
            ));
        }
        Ok(())
    }

    /// The least time the hung step acts within the hang: its lead, the click on the hung target (`press_ms`) with no
    /// pause, the quiet wait, Notepad brought forward (`front_ms`), the click on its line and its pause, the keys with
    /// none, and the strip's wait. The target comes forward before its hang, as a sleeping window cannot take it.
    pub fn hung_acts_ms(&self, front_ms: u64, press_ms: u64) -> u64 {
        [
            press_ms,
            self.hung_quiet_ms,
            front_ms,
            press_ms,
            self.settle_ms,
            self.hung_wait_ms,
        ]
        .iter()
        .fold(self.hang_lead_ms, |sum, ms| sum.saturating_add(*ms))
    }

    /// Refuses a hung app's quiet wait the module's look outlasts, a hang that does not outlast every act in it by
    /// `hang_margin_ms`, and one past target-window's `hang_max_ms`.
    fn check_hang(&self, front_ms: u64, press_ms: u64) -> Result<(), String> {
        let look = look_ms(&selection_defaults()?);
        if self.hung_quiet_ms <= look {
            return Err(format!(
                "g19_product.hung_quiet_ms must outlast the module's look, {look} ms"
            ));
        }
        let acts = self.hung_acts_ms(front_ms, press_ms);
        if self.hang_ms <= acts.saturating_add(self.hang_margin_ms) {
            return Err(format!(
                "g19_product.hang_ms must outlast the acts during the hang, {acts} ms, by hang_margin_ms"
            ));
        }
        let target = kx_target_window::config::load().map_err(|e| e.to_string())?;
        if self.hang_ms > target.hang_max_ms {
            return Err(format!(
                "g19_product.hang_ms must stay within kx-target-window.toml's hang_max_ms ({} ms)",
                target.hang_max_ms
            ));
        }
        Ok(())
    }
}
