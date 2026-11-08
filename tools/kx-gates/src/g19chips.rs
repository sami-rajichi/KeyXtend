//! G19 on the product (`g19 keyxtend`): the selection strip on the test build, in the gate's own Notepad document,
//! Chrome page, target-window and terminals; and `g19 translate-read` and `g19 translate-beside`, which only read the
//! translate page the owner opened.
#![cfg(windows)]

mod act;
mod addr;
mod caret;
mod clear;
mod consent;
mod docs;
mod gate;
mod hides;
mod hung;
mod link;
mod notepad;
mod numlock;
mod ours;
mod page;
mod place;
mod plan;
mod procs;
mod read;
mod script;
mod select;
mod spots;
mod strip;
mod term;
mod terms;

pub use link::{site_key, text_of};
pub use read::{BESIDE, READ};

use std::path::Path;

use serde_json::{Map, Value, json};

use crate::apps::{AppKind, Ctx, Opened};
use crate::freshcopy::Fresh;
use crate::g19chips::gate::Gate;
use crate::g19chips::notepad::Lines;
use crate::g20prod::copy_line;
use crate::g23::notice;
use crate::g25::part;
use crate::out::say;
use crate::product::NAME;
use crate::{clipkeep, probe, win, winclip};

/// The gate's name in its result lines.
pub const GATE: &str = "G19";
/// The cases that must show no strip for `quiet_ms`; the hung app's waits fall within its hang.
pub const QUIET: usize = page::QUIET + hides::QUIET + consent::QUIET;
/// The gate's clicks, keys and drags at the least, each followed by `settle_ms`.
pub const ACTS: usize = notepad::ACTS
    + page::ACTS
    + caret::ACTS
    + ours::ACTS
    + spots::ACTS
    + consent::ACTS
    + terms::ACTS
    + hung::ACTS
    + hides::ACTS;

/// The notes: each case's box kind, the Arabic line's boxes, and the strip on a second screen.
fn notes(cases: &[Value], second: &Value) -> Value {
    let kinds: Map<String, Value> = (cases.iter())
        .filter_map(|c| Some((c["case"].as_str()?.to_string(), c["kind"].clone())))
        .collect();
    let arabic = cases
        .iter()
        .find(|c| c["case"] == consent::arabic_case(0))
        .map(|c| c["placed"].clone());
    json!({ "kinds": kinds, "arabic_line": arabic, "second_screen": second, "ok": true })
}

/// True when the run passed, its data folder went, and nothing it opened or wrote was left behind.
fn passed(run_ok: bool, removed: bool, left: &[String]) -> bool {
    run_ok && removed && left.is_empty()
}

/// Each step on the gate's Notepad document and beside it, the tray last, its window moved only when `own`; `cases`
/// gathers every selection case.
fn steps(
    gate: &Gate,
    (app, own): (&Opened, bool),
    cases: &mut Vec<Value>,
    left: &mut Vec<String>,
) -> Result<Value, String> {
    probe::front(gate.ctx, app)?;
    let lines = Lines::read(gate, app)?;
    let on = |cases: &mut Vec<Value>, f: &dyn Fn(&mut Vec<Value>) -> Result<Value, String>| {
        part(probe::front(gate.ctx, app).and_then(|()| f(cases)))
    };
    let notepad = on(cases, &|c| notepad::edits(gate, app, &lines, c));
    let page = part(page::run(gate, cases, left));
    let edits_ok = notepad["ok"] == true && page["ok"] == true;
    let mut v = json!({ "edits": { "notepad": notepad, "page": page, "ok": edits_ok } });
    v["caret"] = on(cases, &|_| caret::run(gate, app, &lines));
    v["ours"] = on(cases, &|c| ours::run(gate, app, &lines, c));
    v["near_bottom"] = on(cases, &|c| spots::near_bottom(gate, (app, own), &lines, c));
    let second =
        probe::front(gate.ctx, app).map(|()| spots::second_screen(gate, (app, own), &lines));
    v["second"] = second.unwrap_or_else(|e| json!({ "error": e }));
    v["translate"] = on(cases, &|c| consent::run(gate, app, &lines, c));
    v["terminals"] = part(terms::run(gate, cases, left));
    let hung = probe::front(gate.ctx, app).and_then(|()| hung::run(gate, app, &lines, cases, left));
    v["hung"] = part(hung);
    let mut hides = on(cases, &|c| hides::run(gate, app, &lines, c));
    let tray = on(cases, &|c| hides::tray(gate, app, &lines, c));
    hides["ok"] = json!(hides["ok"] == true && tray["ok"] == true);
    hides["tray"] = tray;
    v["hides"] = hides;
    Ok(v)
}

/// The run on a ready copy at `program` with process id `pid`: the start's notices answered, then every step on the
/// gate's own Notepad document, which closes after; `left` gathers the clean-up.
fn drive(ctx: &Ctx, (pid, program): (u32, &Path), left: &mut Vec<String>) -> Result<Value, String> {
    let gate = Gate::new(ctx, (pid, program))?;
    notice::check_normal(ctx, &gate.reader)?;
    let mut cases = Vec::new();
    let before = win::snapshot();
    let doc = probe::open_text(ctx, AppKind::Notepad, &docs::doc(gate.g()))?;
    let own = spots::own(&before, win::key(doc.app.hwnd));
    let mut later = Vec::new();
    let (mut v, notes_left) = probe::run_on(ctx, doc, |d| {
        steps(&gate, (&d.app, own), &mut cases, &mut later)
    })?;
    left.extend(later.into_iter().chain(notes_left));
    v["cases"] = plan::groups(&cases);
    v["each"] = json!(cases);
    let second = v["second"].take();
    v[plan::NOTES] = notes(&cases, &second);
    v["ok"] = json!(plan::passed(&v));
    Ok(v)
}

/// True for a copy the run made: one of its test lines, or a reply of its ping.
fn is_test(ctx: &Ctx, text: &str) -> bool {
    let g = &ctx.cfg.g19_product;
    let t = text.trim();
    [&g.copy_line, &g.cut_line, &g.term_line, &g.page_text]
        .iter()
        .any(|l| l.as_str() == t)
        || t.contains(&g.ping_host)
}

/// Runs `g19 keyxtend`, `translate-read` or `translate-beside`: the first two are refused under uiAccess or while a
/// product copy runs, and their data folder and the run's copies in Windows' clipboard history go at the end.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    match name {
        READ => return read::run(ctx),
        BESIDE => return read::beside(ctx),
        NAME => {}
        _ => {
            return Err(format!(
                "G19 runs on {NAME}, {READ} or {BESIDE}, not {name}"
            ));
        }
    }
    let fresh = Fresh::open(ctx, GATE)?;
    let (program, since, mut left) = (
        fresh.program.display().to_string(),
        winclip::now(),
        Vec::new(),
    );
    let (got, ended) = fresh.with_copy((true, ctx.cfg.g19_product.ready_ms), |pid| {
        clipkeep::keep(ctx.cfg, || drive(ctx, (pid, &fresh.program), &mut left))
    });
    let removed = fresh.close(ended.as_ref());
    let forgot = probe::forget_copies(ctx, since, &|t| is_test(ctx, t));
    let run = copy_line(got, ended.as_ref());
    let pass = passed(run["ok"] == true, removed.is_ok(), &left);
    say!("{GATE} {NAME}: {}", run["ok"]);
    Ok(json!({
        "gate": GATE, "name": NAME, "program": program, "run": run, "clean_up": left, "forgot": forgot,
        "data_removed": removed.is_ok(), "data_error": removed.err(), "pass": pass,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_run_length_counts_every_quiet_case() {
        assert_eq!(
            QUIET, 6,
            "the password and unknown boxes, the aimed panel, the tray, two Not nows; the hung app's is in its hang"
        );
    }

    #[test]
    fn the_notes_name_each_cases_box_kind_the_arabic_lines_boxes_and_the_second_screen() {
        let cases = [
            json!({ "case": "notepad_keys", "kind": "RichEditD2DPT / type 50030" }),
            json!({ "case": consent::arabic_case(0), "kind": "x", "placed": { "boxes": 2, "one_line": true } }),
        ];
        let n = notes(&cases, &json!({ "screens": 2, "placed": { "ok": true } }));
        assert_eq!(n["kinds"]["notepad_keys"], "RichEditD2DPT / type 50030");
        assert_eq!(n["arabic_line"]["one_line"], true);
        assert_eq!(n["second_screen"]["placed"]["ok"], true);
    }

    #[test]
    fn anything_left_behind_fails_the_run() {
        assert!(passed(true, true, &[]));
        assert!(!passed(
            true,
            true,
            &["left open: [Notepad] of notepad.exe".into()]
        ));
        assert!(!passed(true, false, &[]), "the data folder stayed");
        assert!(!passed(false, true, &[]));
    }
}
