//! G20's keys in the gate's own Notepad tab: Ctrl+A and Ctrl+C for the skipped app's copy, and Ctrl+A then Insert's
//! click, which pastes there. Each goes out through `ownkeys`, only while that tab shows.
#![cfg(windows)]

use crate::apps::Opened;
use crate::config::GatesConfig;
use crate::ownkeys::{self, Desk};
use crate::winclip::line;

/// Ctrl+A then Ctrl+C in `app` on `desk`, each only while the gate's own tab shows.
pub fn copy_in_tab(desk: &dyn Desk, app: &Opened, cfg: &GatesConfig) -> Result<(), String> {
    for combo in [&cfg.keys.select_all, &cfg.keys.copy] {
        ownkeys::send(desk, app, combo)?;
        desk.settle(cfg.timing.key_settle_ms);
    }
    Ok(())
}

/// Ctrl+A in `app` on `desk`, then Insert's `click`, which pastes there; each only while the gate's own tab shows.
pub fn paste(
    desk: &dyn Desk,
    app: &Opened,
    cfg: &GatesConfig,
    click: &dyn Fn() -> Result<(), String>,
) -> Result<(), String> {
    ownkeys::send(desk, app, &cfg.keys.select_all)?;
    desk.settle(cfg.timing.key_settle_ms);
    ownkeys::click(desk, app, click)
}

/// Whether Notepad's copy of `want` happened: the clipboard `changed`, and its text `got`, line ends cut, is `want`.
pub fn skipped_ok(changed: bool, got: Option<&str>, want: &str) -> Result<(), String> {
    if !changed {
        return Err("Notepad's copy never reached the clipboard".to_string());
    }
    if got.map(line) != Some(want) {
        return Err("the clipboard does not hold Notepad's copy of the skipped text".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
