//! G23 on the product (`g23 keyxtend`): the test build on a data folder the gate makes, a test Login saved through
//! Quick-fill with clicks only, then Both, Password and User, each typed from the no-focus keyboard into a fresh Chrome
//! login page with the panel left open, until its close button shuts it. Only the wanted text may arrive; nothing may
//! reach the clipboard, and no Enter or submit may happen.
#![cfg(windows)]

mod fills;
pub(crate) mod live;
pub(crate) mod notice;
mod page;
mod panel;
mod seed;

pub(crate) use notice::OFFER_ASK;

use kx_platform_windows::uia::Uia;
use serde_json::{Value, json};

use crate::apps::{Ctx, Opened};
use crate::freshcopy::Fresh;
use crate::g20prod::copy_line;
use crate::g23::fills::Fill;
use crate::g23::panel::Panel;
use crate::out::say;
use crate::product::NAME;
use crate::{g25, probe, typebox};

/// The gate's name in its result lines.
const GATE: &str = "G23";
/// The seed's part in the result line.
const SEED: &str = "seed";
/// Whether the layout in front came back after the seed's typing, in the result line.
const LAYOUT_BACK: &str = "layout_back";
/// The panel's close button after the last fill, in the result line.
const SHUT: &str = "shut";
/// The fills, in order: Both on the seed's page, then each one-part fill on a fresh page of its own.
const FILLS: [Fill; 3] = [Fill::Both, Fill::Password, Fill::User];

/// A login page's run: what `drive` gave on the page in front, and what closing the page left.
type OnPage<T> = Result<(T, Vec<String>), String>;

/// Runs `drive` on a fresh login page in front, then closes the page.
fn on_page<T>(ctx: &Ctx, drive: impl FnOnce(&Opened) -> Result<T, String>) -> OnPage<T> {
    let doc = probe::open_page(ctx, &page::page(ctx))?;
    probe::run_on(ctx, doc, |doc| {
        probe::front(ctx, &doc.app)?;
        drive(&doc.app)
    })
}

/// Saves the test entry on the first page, then runs Both there: the seed's part, Both's, and whether the layout in
/// front came back after the typing.
fn first(ctx: &Ctx, panel: &Panel, uia: &Uia, app: &Opened) -> Result<Value, String> {
    let (seeded, back) = typebox::typed_in(ctx, app, |t| seed::seed(ctx, panel, t));
    let seed = seeded?;
    let both = FILLS[0];
    let part = g25::part(fills::run(ctx, panel, uia, app, both));
    Ok(json!({ SEED: seed, both.name(): part, LAYOUT_BACK: back }))
}

/// The run on a ready copy: a normal run, the seed and Both, then each one-part fill with the panel still open, and its
/// close button last; `left` gathers the clean-up.
fn drive(ctx: &Ctx, left: &mut Vec<String>) -> Result<Value, String> {
    let panel = Panel::new(ctx)?;
    notice::check_normal(ctx, panel.reader())?;
    let uia = Uia::new().map_err(|e| e.to_string())?;
    let (mut v, notes) = on_page(ctx, |app| first(ctx, &panel, &uia, app))?;
    left.extend(notes);
    for &fill in &FILLS[1..] {
        let got = on_page(ctx, |app| fills::run(ctx, &panel, &uia, app, fill));
        let part = got.map(|(part, notes)| {
            left.extend(notes);
            part
        });
        v[fill.name()] = g25::part(part);
    }
    v[SHUT] = g25::part(panel.close().map(|went| json!({ "ok": went })));
    v["ok"] = json!(passed(&v));
    Ok(v)
}

/// True when the seed, every fill, the panel's close button and the layout's return passed.
fn passed(v: &Value) -> bool {
    let mut parts = [SEED, SHUT]
        .into_iter()
        .chain(FILLS.iter().map(|f| f.name()));
    parts.all(|p| v[p]["ok"] == true) && v[LAYOUT_BACK] == true
}

/// Runs G23 on the product, refused under uiAccess or while a product copy runs; the data folder goes at the end.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    if name != NAME {
        return Err(format!("G23 runs on {NAME}, not {name}"));
    }
    let fresh = Fresh::open(ctx, GATE)?;
    let program = fresh.program.display().to_string();
    let mut left = Vec::new();
    let (got, ended) = fresh.with_copy((false, ctx.cfg.g23.ready_ms), |_| drive(ctx, &mut left));
    let removed = fresh.close(ended.as_ref());
    let run = copy_line(got, ended.as_ref());
    let pass = run["ok"] == true && removed.is_ok();
    say!("{GATE} {NAME}: {}", run["ok"]);
    Ok(json!({
        "gate": GATE, "name": NAME, "program": program, "run": run, "clean_up": left,
        "data_removed": removed.is_ok(), "data_error": removed.err(), "pass": pass,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A run whose every part passed.
    fn good() -> Value {
        let yes = || json!({ "ok": true });
        json!({ "seed": yes(), "both": yes(), "password": yes(), "user": yes(), "shut": yes(), "layout_back": true })
    }

    #[test]
    fn the_run_passes_when_every_part_did() {
        assert!(passed(&good()));
        assert_eq!(FILLS.len(), crate::p9cfg::FILLS, "the run-length count");
    }

    #[test]
    fn the_run_fails_when_any_part_failed_or_is_missing() {
        for part in ["seed", "both", "password", "user", "shut"] {
            let mut v = good();
            v[part] = json!({ "ok": false });
            assert!(!passed(&v), "{part} failed");
            v[part] = json!({ "error": "no panel", "ok": false });
            assert!(!passed(&v), "{part} broke off");
            let mut gone = good();
            gone.as_object_mut().expect("an object").remove(part);
            assert!(!passed(&gone), "{part} missing");
        }
        let mut v = good();
        v["layout_back"] = json!(false);
        assert!(!passed(&v), "the layout stayed");
    }
}
