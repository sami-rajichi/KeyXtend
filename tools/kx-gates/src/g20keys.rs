//! G20's keys on the product's keyboard: the clip key that opens and shuts our panel, the Esc key that clears its
//! search, and the Win and V keys, which open our panel with the history key on and Windows' own with it off.
#![cfg(windows)]

use kx_mod_keyboard::{Latch, Nav, Role};
use serde_json::{Value, json};

use crate::apps::{Ctx, Opened};
use crate::clippanel::Panel;
use crate::g25watch::Watch;
use crate::g25wins::{self, EscTo};
use crate::modekey::ModeKey;
use crate::ownkeys::{self, Real};
use crate::product::Keyboard;
use crate::win::sleep_ms;
use crate::{board, g25};

/// Clicks the product's key `id` only while `app` is in front with our tab, so the keyboard acts for it alone.
fn tap(app: &Opened, id: &str) -> Result<(), String> {
    ownkeys::click(&Real, app, &|| Keyboard::find()?.click(id))
}

/// Opens our panel with the clip key, or shuts it, unless it is so already, and waits for the panel to follow.
pub fn clip_key(app: &Opened, panel: &Panel, open: bool) -> Result<(), String> {
    if panel.window().is_some() == open {
        return Ok(());
    }
    tap(app, &board::id_of(Role::Clip)?)?;
    panel.wait(open)
}

/// Clicks the product's Esc key, which clears the panel's search, or shuts the panel when the search is empty.
pub fn esc(app: &Opened) -> Result<(), String> {
    tap(app, &board::nav_key(Nav::Esc)?)
}

/// Our panel's part of the Win and V check: it opened, no window of `[g25] panel_classes` showed, and the clip key
/// shut it; a Windows window that showed was closed with Esc.
fn ours_line(
    ctx: &Ctx,
    app: &Opened,
    panel: &Panel,
    watch: &Watch,
    opened: &Result<(), String>,
) -> Value {
    sleep_ms(ctx.cfg.g20_product.settle_ms);
    let theirs = !watch.shown().is_empty();
    let theirs_closed =
        theirs.then(|| g25::part(g25wins::closes(ctx, app, watch, EscTo::WatchedOrApp)));
    let shut = opened
        .as_ref()
        .map_err(Clone::clone)
        .and_then(|()| clip_key(app, panel, false));
    json!({
        "ours_opened": opened.is_ok(), "open_error": opened.as_ref().err(), "theirs_shown": theirs,
        "theirs_closed": theirs_closed, "ours_shut": shut.is_ok(), "shut_error": shut.as_ref().err(),
        "ok": opened.is_ok() && !theirs && shut.is_ok(),
    })
}

/// The product's Win and V keys with the history key on, from a closed panel: our panel opens instead of Windows'
/// history, and the clip key shuts it. Win and its spare Shift are let go after, and the part passes once both are.
pub fn ours(ctx: &Ctx, app: &Opened, panel: &Panel) -> Result<Value, String> {
    // The panel starts closed, so only Win and V can open it.
    clip_key(app, panel, false)?;
    let g = &ctx.cfg.g25;
    let (win_id, v) = (board::latch_id(Latch::Win)?, board::char_key(g.v_scan)?);
    let win_key = ModeKey::by_id(win_id.clone())?;
    let watch = Watch::start(&g.panel_classes, None);
    let typed = ownkeys::click(&Real, app, &|| win_key.set(true, g25::wait(ctx)))
        .and_then(|()| ownkeys::click(&Real, app, &|| win_key.keyboard.click(&v)));
    let opened = typed.and_then(|()| panel.wait(true));
    let mut result = ours_line(ctx, app, panel, &watch, &opened);
    let after = g25wins::let_go(&win_id).unwrap_or((None, None));
    result["win_let_go"] = json!(after.0 == Some(false));
    result["shift_let_go"] = json!(after.1 == Some(false));
    result["ok"] = json!(result["ok"] == true && g25wins::both_let_go(after));
    Ok(result)
}

/// The product's Win and V keys with the history key off: Windows' own history opens and Esc closes it, as in G25.
/// Windows opens it only for a uiAccess product, so G20 skips this part on the test build.
pub fn theirs(ctx: &Ctx, app: &Opened) -> Result<Value, String> {
    g25wins::clip_window(ctx, app)
}
