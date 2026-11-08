//! The key and wait checks of `[g19_product]`.
#![cfg(windows)]

use windows::Win32::UI::Input::KeyboardAndMouse::VK_NUMLOCK;

use crate::apps::INPUT;
use crate::p5cfg::SEP;
use crate::p8cfg::wait;
use crate::p10cfg::{G19Product, caret_ms, selection_defaults};

impl G19Product {
    /// Refuses a Num Lock that is another key, no Shift+Home or caret key, a browser that is a path, and a console
    /// that would run no script of the gate's.
    pub(super) fn check_keys(&self) -> Result<(), String> {
        if self.numlock != [VK_NUMLOCK.0] {
            return Err("g19_product.numlock must be the Num Lock key alone".into());
        }
        if self.select_home.is_empty() {
            return Err("g19_product.select_home must hold a key".into());
        }
        if self.caret_left.is_empty() {
            return Err("g19_product.caret_left must hold a key".into());
        }
        let named = self
            .tray
            .split_once(SEP)
            .is_some_and(|(a, b)| !a.is_empty() && !b.is_empty());
        if !named {
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

    /// Refuses a wait outside 1 ms to `most_ms`, a hang the module's look outlasts or that leads into nothing, a
    /// negative slack, and a caret slack as long as the caret's Paste stays.
    pub(super) fn check_waits(&self, most_ms: u64) -> Result<(), String> {
        let waits = [
            ("ready_ms", self.ready_ms),
            ("strip_wait_ms", self.strip_wait_ms),
            ("gone_wait_ms", self.gone_wait_ms),
            ("quiet_ms", self.quiet_ms),
            ("settle_ms", self.settle_ms),
            ("hang_ms", self.hang_ms),
            ("hang_lead_ms", self.hang_lead_ms),
            ("ping_ms", self.ping_ms),
            ("caret_slack_ms", self.caret_slack_ms),
            ("owner_wait_ms", self.owner_wait_ms),
        ];
        let key = |k: &str| format!("g19_product.{k}");
        waits
            .iter()
            .try_for_each(|(k, ms)| wait(&key(k), *ms, most_ms))?;
        let s = selection_defaults()?;
        let look = u64::from(s.settle_ms).saturating_add(u64::from(s.read_ms));
        if self.hang_ms <= look {
            return Err(format!(
                "g19_product.hang_ms must outlast the module's look, {look} ms"
            ));
        }
        if self.hang_lead_ms >= self.hang_ms {
            return Err("g19_product.hang_lead_ms must be shorter than hang_ms".into());
        }
        if self.place_slack_px < 0 {
            return Err("g19_product.place_slack_px must not be negative".into());
        }
        if self.caret_slack_ms >= caret_ms(&s) {
            return Err(
                "g19_product.caret_slack_ms must be shorter than the module's caret_s".into(),
            );
        }
        Ok(())
    }
}
