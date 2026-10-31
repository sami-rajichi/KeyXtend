//! G1 read-back: the text an app holds, from its saved file, its log or the clipboard. Every key goes through
//! `ownkeys`, so the owner's tabs are never saved or read.
#![cfg(windows)]

use kx_platform_windows::clip::{self, open};

use crate::apps::{AppKind, Ctx, Opened};
use crate::config::GatesConfig;
use crate::ownkeys::{self, Desk, Real};
use crate::win;
use crate::{clipkeep, out, tlog};

/// Byte-order mark some editors put first.
const BOM: char = '\u{FEFF}';

/// Ctrl+A, Ctrl+C in `app` on `desk`, then the clipboard text.
fn copy(cfg: &GatesConfig, desk: &dyn Desk, app: &Opened) -> Result<String, String> {
    let t = &cfg.timing;
    let before = open::sequence();
    ownkeys::send(desk, app, &cfg.keys.select_all)?;
    desk.settle(t.key_settle_ms);
    ownkeys::send(desk, app, &cfg.keys.copy)?;
    if !open::wait_change(before, clipkeep::wait(t)) {
        return Err("the clipboard did not change after Ctrl+C".to_string());
    }
    clip::read_text(clipkeep::wait(t), cfg.g1.max_read_chars)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no text on the clipboard after Ctrl+C".to_string())
}

/// Reads back what the app holds; its keys go out only while it is in front with our tab shown.
pub fn read_back(ctx: &Ctx, app: &Opened) -> Result<String, String> {
    read_on(ctx, &Real, app)
}

/// `read_back` on `desk`.
pub(crate) fn read_on(ctx: &Ctx, desk: &dyn Desk, app: &Opened) -> Result<String, String> {
    let (cfg, t) = (ctx.cfg, &ctx.cfg.timing);
    if app.kind == AppKind::Target {
        let log = &ctx.target.log;
        let text = std::fs::read_to_string(log).map_err(|e| format!("{}: {e}", log.display()))?;
        return Ok(tlog::decode(&tlog::parse(&text)));
    }
    let key = match app.kind {
        AppKind::Notepad => &cfg.keys.save,
        AppKind::Terminal => &cfg.keys.enter,
        // The owner's clipboard is saved first and put back after.
        _ => return clipkeep::keep(cfg, || copy(cfg, desk, app)),
    };
    let file = app.file.as_deref().ok_or("no file to read")?;
    ownkeys::send(desk, app, key)?;
    let text = out::wait_read(file, t.read_wait_ms, t.poll_ms)?;
    Ok(text
        .strip_prefix(BOM)
        .map_or_else(|| text.clone(), String::from))
}

/// Saves the app's file and waits until its text passes `want`, since the first read may catch the old save.
pub fn saved_until(ctx: &Ctx, app: &Opened, want: impl Fn(&str) -> bool) -> Result<bool, String> {
    saved_on(ctx, &Real, app, want)
}

/// `saved_until` on `desk`.
pub(crate) fn saved_on(
    ctx: &Ctx,
    desk: &dyn Desk,
    app: &Opened,
    want: impl Fn(&str) -> bool,
) -> Result<bool, String> {
    if want(&read_on(ctx, desk, app)?) {
        return Ok(true);
    }
    let (t, file) = (
        &ctx.cfg.timing,
        app.file.as_deref().ok_or("no file to read")?,
    );
    let now = || {
        let text = std::fs::read_to_string(file).ok()?;
        want(text.strip_prefix(BOM).unwrap_or(&text)).then_some(())
    };
    Ok(win::poll_until(t.read_wait_ms, t.poll_ms, now).is_some())
}

#[cfg(test)]
mod tests;
