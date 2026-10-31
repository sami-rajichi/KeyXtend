//! G20 on the product (`g20 keyxtend`): the test build on its own data folder with the clipboard defaults, the gate's
//! copies in its panel, searches and ticks on its keys, Insert into the gate's own Notepad tab, then Win and V with the
//! history key on, and off only on a uiAccess product. It overwrites the clipboard and Windows' history, and leaves
//! Insert's last entry on it.
#![cfg(windows)]

use std::path::Path;

use serde_json::{Value, json};

use crate::apps::{AppKind, Ctx, Opened};
use crate::clipcopy::ClipSession;
use crate::copyend::{self, Ended};
use crate::g8start::Started;
use crate::g20steps::Steps;
use crate::out::say;
use crate::product::NAME;
use crate::winclip::{self, line};
use crate::winpanel::{SKIPPED, history_off, judged_ok, skipped};
use crate::{clipkeep, g20keys, g25, probe, typebox, workertok};

/// The copies G20 reports: the history key on, then off, which starts only on a uiAccess product.
pub const COPIES: usize = 2;
/// Why the second copy was not started.
const STILL_RUNS: &str =
    "the first copy could not be ended, so the copy with the history key off was not started";

/// The first copy's steps: its rows, searches, Insert, Win and V, and the worker check, which fails only on a worker
/// with too many rights.
fn first(ctx: &Ctx, app: &Opened, s: &Started, program: &Path) -> Result<Value, String> {
    probe::front(ctx, app)?;
    let steps = Steps::new(ctx, app)?;
    steps.copy_all()?;
    let rows = steps.rows()?;
    let (typed, layout_back) = typebox::typed_in(ctx, app, |t| Ok(steps.searches(t)));
    let (searches, ticks) = typed?;
    let insert = g25::part(steps.insert(&ticks));
    let win_v = g25::part(g20keys::ours(ctx, app, &steps.panel));
    let worker = workertok::check(s.process.pid(), program);
    let all = |v: &[&Value]| v.iter().all(|p| p["ok"] == true);
    let found = searches.iter().all(|p| p["ok"] == true);
    let ok = all(&[&rows, &insert, &win_v]) && found && layout_back && worker["fails"] != true;
    Ok(json!({
        "rows": rows, "searches": searches, "layout_back": layout_back, "insert": insert, "win_v": win_v,
        "worker": worker, "ok": ok,
    }))
}

/// The second copy's step: Windows' own history from the product's Win and V keys, which needs uiAccess.
fn second(ctx: &Ctx, app: &Opened) -> Result<Value, String> {
    probe::front(ctx, app)?;
    g20keys::theirs(ctx, app)
}

/// A copy's part for the result: what its steps gave, or their error, and how the copy went; it passes only when
/// both did.
pub(crate) fn copy_line(got: Result<Value, String>, ended: Option<&Ended>) -> Value {
    let mut v = got.unwrap_or_else(|e| json!({ "error": e, "ok": false }));
    let ok = v["ok"] == true && ended.is_some_and(Ended::ok);
    v["copy"] = json!(ended.map(Ended::line));
    v["ok"] = json!(ok);
    v
}

/// Empties the clipboard, so a copy starts on none of the owner's.
fn empty(ctx: &Ctx) -> Result<(), String> {
    clipkeep::write_text(None, &[], clipkeep::wait(&ctx.cfg.timing)).map_err(|e| e.to_string())
}

/// One copy with the history key `on`, started on an emptied clipboard: what `drive` gave and how the copy went.
fn copy(
    ctx: &Ctx,
    session: &ClipSession,
    on: bool,
    drive: impl FnOnce(&Started) -> Result<Value, String>,
) -> (Result<Value, String>, Option<Ended>) {
    match empty(ctx) {
        Ok(()) => session.with_copy(on, ctx.cfg.g20_product.ready_ms, drive),
        Err(e) => (Err(e), None),
    }
}

/// Both copies on `app`, the second only once the first ended, and skipped when the first read a product without
/// uiAccess: each one's part.
fn both(ctx: &Ctx, session: &ClipSession, app: &Opened) -> [Value; COPIES] {
    let (got, ended) = copy(ctx, session, true, |s| first(ctx, app, s, &session.program));
    let one = copy_line(got, ended.as_ref());
    if !copyend::may_follow(ended.map(|e| e.gone)) {
        return [one, copy_line(Err(STILL_RUNS.to_string()), None)];
    }
    let two = history_off(workertok::product_of(&one["worker"]), || {
        let (got, ended) = copy(ctx, session, false, |_| second(ctx, app));
        copy_line(got, ended.as_ref())
    });
    [one, two]
}

/// A part's word on the console: whether it passed, or that it was skipped.
fn said(part: &Value) -> String {
    if skipped(part) {
        SKIPPED.to_string()
    } else {
        part["ok"].to_string()
    }
}

/// True when at least one copy's part was judged, every judged one passed and the test build's settings came back;
/// a skipped part neither fails nor passes the gate.
pub(crate) fn passed(parts: &[Value], back: bool) -> bool {
    judged_ok(parts) && back
}

/// Runs G20 on the product, refused under uiAccess or while a product copy runs. Windows' history then forgets the
/// gate's own copies, and the test build's settings and history come back as they were.
pub fn run(ctx: &Ctx) -> Result<Value, String> {
    let g = &ctx.cfg.g20_product;
    let session = ClipSession::open(ctx)?;
    let (program, since) = (session.program.clone(), winclip::now());
    let driven = probe::open_text(ctx, AppKind::Notepad, &g.skipped)
        .and_then(|doc| probe::run_on(ctx, doc, |doc| Ok(both(ctx, &session, &doc.app))));
    let ours: Vec<&str> = g.texts().into_iter().chain(g.hidden()).collect();
    let is_test = |t: &str| ours.contains(&line(t));
    let history = probe::forget_copies(ctx, since, &is_test);
    let recovered = session.recovered;
    let back = session.close();
    let (parts, left) = driven?;
    let pass = passed(&parts, back.is_ok());
    let [first, second] = parts;
    say!(
        "G20 {NAME}: first copy {}, second copy {}",
        said(&first),
        said(&second)
    );
    Ok(json!({
        "gate": "G20", "name": NAME, "program": program.display().to_string(), "first": first, "second": second,
        "history": history, "clean_up": left, "settings_put_back": back.is_ok(), "settings_error": back.err(),
        "settings_recovered": recovered, "pass": pass,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::copyend::Gone;
    use crate::winpanel::NO_UI_ACCESS;
    use crate::workertok::{PRODUCT, Rights};

    fn ended(gone: Gone) -> Ended {
        Ended {
            gone,
            note: None,
            released: Vec::new(),
        }
    }

    #[test]
    fn a_copy_passes_only_when_its_steps_passed_and_it_quit_by_itself() {
        let good = || Ok(json!({ "ok": true }));
        assert_eq!(copy_line(good(), Some(&ended(Gone::Quit)))["ok"], true);
        assert_eq!(
            copy_line(good(), Some(&ended(Gone::Crashed(Some(1)))))["ok"],
            false
        );
        assert_eq!(copy_line(good(), None)["ok"], false, "no copy");
        let failed = copy_line(Err("no panel".into()), Some(&ended(Gone::Quit)));
        assert_eq!(
            (failed["ok"].as_bool(), failed["error"].as_str()),
            (Some(false), Some("no panel"))
        );
        assert_eq!(
            copy_line(Ok(json!({ "ok": false })), Some(&ended(Gone::Quit)))["ok"],
            false
        );
    }

    #[test]
    fn the_gate_passes_when_both_copies_passed_and_the_settings_came_back() {
        let (yes, no) = (json!({ "ok": true }), json!({ "ok": false }));
        assert!(passed(&[yes.clone(), yes.clone()], true));
        assert!(!passed(&[yes.clone(), yes.clone()], false), "settings");
        assert!(!passed(&[yes, no], true));
        assert!(!passed(&[], true));
    }

    /// The first copy's worker part with the product's token as the test build gives it, or with uiAccess.
    fn worker(ui_access: bool) -> Value {
        let product = Rights {
            ui_access,
            level: workertok::MEDIUM,
        };
        json!({ PRODUCT: product, "workers": [], "ok": false, "fails": false })
    }

    #[test]
    fn a_product_without_ui_access_skips_the_history_off_part_and_the_gate_can_still_pass() {
        let off = history_off(workertok::product_of(&worker(false)), || {
            panic!("the history-off part ran on a product without uiAccess")
        });
        assert_eq!(off[SKIPPED], true);
        assert_eq!(off["why"], NO_UI_ACCESS);
        let (yes, no) = (json!({ "ok": true }), json!({ "ok": false }));
        assert!(
            passed(&[yes, off.clone()], true),
            "a skipped part fails nothing"
        );
        assert!(!passed(&[no, off.clone()], true), "nor passes anything");
        assert!(!passed(&[off.clone(), off], true), "nothing was judged");
    }

    #[test]
    fn a_ui_access_product_still_runs_the_history_off_part_and_judges_it() {
        let yes = json!({ "ok": true });
        let product = workertok::product_of(&worker(true));
        let failed = history_off(product, || json!({ "ok": false }));
        assert_eq!(failed["ok"], false, "{failed}");
        assert!(!passed(&[yes.clone(), failed], true));
        let good = history_off(product, || json!({ "ok": true }));
        assert!(passed(&[yes.clone(), good], true));
        let unread = history_off(None, || json!({ "ok": false }));
        assert!(
            !passed(&[yes, unread], true),
            "an unread token still runs it"
        );
    }
}
