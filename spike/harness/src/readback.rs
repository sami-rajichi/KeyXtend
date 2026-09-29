//! G1 read-back: the text an app holds, from its saved file, its log or the clipboard.

use spike_core::window::foreground;

use crate::apps::{AppKind, Ctx, Opened};
use crate::config::HarnessConfig;
use crate::win::{self, sleep_ms};
use crate::{clip, clipkeep, keys, out, tlog};

/// Byte-order mark some editors put first.
const BOM: char = '\u{FEFF}';

/// Ctrl+A, Ctrl+C, then the clipboard text.
fn copy(cfg: &HarnessConfig) -> Result<String, String> {
    let t = &cfg.timing;
    let before = clip::sequence();
    keys::combo(&cfg.keys.select_all)?;
    sleep_ms(t.key_settle_ms);
    keys::combo(&cfg.keys.copy)?;
    if !clip::wait_change(before, t.read_wait_ms, t.poll_ms) {
        return Err("the clipboard did not change after Ctrl+C".to_string());
    }
    clip::read_text(t.read_wait_ms, t.poll_ms, cfg.g1.max_read_chars)?
        .ok_or_else(|| "no text on the clipboard after Ctrl+C".to_string())
}

/// Copies all the app's text; the owner's clipboard is saved first and put back after.
fn copy_all(cfg: &HarnessConfig) -> Result<String, String> {
    clipkeep::keep(cfg, || copy(cfg))
}

/// Reads back what the app holds; checks it is still in front before any shortcut.
pub fn read_back(ctx: &Ctx, app: &Opened) -> Result<String, String> {
    let (cfg, t) = (ctx.cfg, &ctx.cfg.timing);
    if app.kind == AppKind::Target {
        let log = &ctx.target.log;
        let text = std::fs::read_to_string(log).map_err(|e| format!("{}: {e}", log.display()))?;
        return Ok(tlog::decode(&tlog::parse(&text)));
    }
    let front = foreground();
    if front != app.hwnd {
        return Err(format!(
            "not in front before reading back; front is {}",
            win::describe(front)
        ));
    }
    let key = match app.kind {
        AppKind::Notepad => &cfg.keys.save,
        AppKind::Terminal => &cfg.keys.enter,
        _ => return copy_all(cfg),
    };
    let file = app.file.as_deref().ok_or("no file to read")?;
    keys::combo(key)?;
    let text = out::wait_read(file, t.read_wait_ms, t.poll_ms)?;
    Ok(text
        .strip_prefix(BOM)
        .map_or_else(|| text.clone(), String::from))
}

/// Saves the app's file and waits until its text passes `want`, since the first read may catch the old save.
pub fn saved_until(ctx: &Ctx, app: &Opened, want: impl Fn(&str) -> bool) -> Result<bool, String> {
    if want(&read_back(ctx, app)?) {
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
