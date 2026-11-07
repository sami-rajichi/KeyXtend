//! G23's test entry, saved through the panel with clicks only: the `fill` key, New, the Login chip, then the name, the
//! user and the password, each line clicked and its letters typed on the keyboard's keys, then Save. The first backup
//! offer is answered Later, and the panel is shut.
#![cfg(windows)]

use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::g23::notice;
use crate::g23::panel::{Panel, fill_key};
use crate::g27type::Typist;
use crate::win::sleep_ms;

/// Saves the test entry while `t`'s app is in front: whether Save closed the New page, and whether the backup offer
/// came and was answered.
pub fn seed(ctx: &Ctx, panel: &Panel, t: &Typist) -> Result<Value, String> {
    let g = &ctx.cfg.g23;
    let layout = t.switch(&g.lang)?;
    fill_key(t.app(), panel, true)?;
    panel.click(&g.id(&g.new))?;
    panel.click(&g.kind_id())?;
    let lines = [
        (g.id(&g.name_line), &g.name),
        (g.value_id(g.user_field), &g.user),
        (g.value_id(g.password_field), &g.password),
    ];
    for (line, text) in &lines {
        panel.click(line)?;
        t.type_in(layout, text)?;
        sleep_ms(g.settle_ms);
    }
    let save = g.id(&g.save);
    panel.click(&save)?;
    let saved = panel.goes(&save)?;
    let offer = notice::answer_offer(ctx, panel.reader())?;
    fill_key(t.app(), panel, false)?;
    Ok(json!({ "saved": saved, "offer_answered": offer, "ok": saved }))
}
