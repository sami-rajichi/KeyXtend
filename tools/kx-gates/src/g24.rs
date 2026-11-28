//! G24: Snip freezes the screen, and what it copies and records is exact; it runs on the product in `g24prod`.
#![cfg(windows)]

use serde_json::Value;

use crate::apps::Ctx;
use crate::g24prod::{self, Run};
use crate::product::NAME;

/// G24's trap part alone (ADR-0059): every window of ours over the frozen screens, and each way out, in about a minute.
pub const TRAP: &str = "trap";

/// Runs G24 on the product, or its trap part alone; the spike face's snip left in P12.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    match name {
        NAME => g24prod::run(ctx, Run::All),
        TRAP => g24prod::run(ctx, Run::Trap),
        _ => Err(format!("G24 runs on {NAME} or {TRAP}, not {name}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frozen_face_is_refused_before_anything_starts() {
        let cfg = crate::config::load().expect("settings load");
        let spike = spike_core::config::load().expect("spike settings");
        let target = kx_target_window::config::load().expect("target settings");
        let ctx = Ctx {
            cfg: &cfg,
            spike: &spike,
            target: &target,
            out: std::env::temp_dir(),
            base: String::new(),
        };
        let e = run(&ctx, spike_core::config::FACE).expect_err("the face is gone");
        assert!(e.contains(NAME) && e.contains(TRAP), "{e}");
    }
}
