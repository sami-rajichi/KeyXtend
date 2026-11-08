//! G19 on the product (`g19 keyxtend`): the selection strip on the test build, in the gate's own Notepad document,
//! Chrome page, target-window and terminals; and `g19 translate-read`, which only reads the translate page the owner
//! opened.
#![cfg(windows)]

mod act;
mod addr;
mod caret;
mod consent;
mod copy;
mod docs;
mod gate;
mod hides;
mod hung;
mod link;
mod notepad;
mod ours;
mod page;
mod place;
mod plan;
mod procs;
mod read;
mod script;
mod strip;
mod term;
mod terms;

pub use link::{site_key, text_of};
pub use read::READ;

use kx_platform::ui::Screens;
use kx_platform_windows::screen::WinScreens;
use serde_json::{Map, Value, json};

use crate::apps::{AppKind, Ctx, Opened};
use crate::g19chips::copy::Fresh;
use crate::g19chips::gate::Gate;
use crate::g19chips::notepad::Lines;
use crate::g20prod::copy_line;
use crate::g23::notice;
use crate::g25::part;
use crate::out::say;
use crate::product::NAME;
use crate::{clipkeep, probe, winclip};

/// The cases that must show no strip for `quiet_ms`.
pub const QUIET: usize = page::QUIET + hides::QUIET + hung::QUIET + consent::QUIET;
/// The gate's clicks, keys and drags at the least, each followed by `settle_ms`.
pub const ACTS: usize = notepad::ACTS
    + page::ACTS
    + caret::ACTS
    + ours::ACTS
    + consent::ACTS
    + terms::ACTS
    + hung::ACTS
    + hides::ACTS;

/// The notes: each case's box kind, the Arabic line's boxes, and the screens.
fn notes(cases: &[Value]) -> Value {
    let kinds: Map<String, Value> = (cases.iter())
        .filter_map(|c| Some((c["case"].as_str()?.to_string(), c["kind"].clone())))
        .collect();
    let arabic = cases
        .iter()
        .find(|c| c["case"] == "arabic_0")
        .map(|c| c["placed"].clone());
    let screens = WinScreens.work_areas().len();
    json!({ "kinds": kinds, "arabic_line": arabic, "screens": screens, "ok": true })
}

/// Each step on the gate's Notepad document and beside it, the tray last; `cases` gathers every selection case.
fn steps(
    gate: &Gate,
    app: &Opened,
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

/// The run on a ready copy with process id `pid`: the start's notices answered, then every step on the gate's own
/// Notepad document, which closes after; `left` gathers the clean-up.
fn drive(ctx: &Ctx, pid: u32, left: &mut Vec<String>) -> Result<Value, String> {
    let gate = Gate::new(ctx, pid)?;
    notice::check_normal(ctx, &gate.reader)?;
    let mut cases = Vec::new();
    let doc = probe::open_text(ctx, AppKind::Notepad, &docs::doc(gate.g()))?;
    let mut later = Vec::new();
    let (mut v, notes_left) =
        probe::run_on(ctx, doc, |d| steps(&gate, &d.app, &mut cases, &mut later))?;
    left.extend(later.into_iter().chain(notes_left));
    v["cases"] = plan::groups(&cases);
    v["each"] = json!(cases);
    v[plan::NOTES] = notes(&cases);
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

/// Runs `g19 keyxtend` or `g19 translate-read`: refused under uiAccess or while a product copy runs; the data folder
/// and the run's copies in Windows' clipboard history go at the end.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    if name == READ {
        return read::run(ctx);
    }
    if name != NAME {
        return Err(format!("G19 runs on {NAME} or {READ}, not {name}"));
    }
    let fresh = Fresh::open(ctx)?;
    let (program, since, mut left) = (
        fresh.program.display().to_string(),
        winclip::now(),
        Vec::new(),
    );
    let (got, ended) = fresh.with_copy(true, |pid| {
        clipkeep::keep(ctx.cfg, || drive(ctx, pid, &mut left))
    });
    let removed = fresh.close(ended.as_ref());
    let forgot = probe::forget_copies(ctx, since, &|t| is_test(ctx, t));
    let run = copy_line(got, ended.as_ref());
    let pass = run["ok"] == true && removed.is_ok();
    say!("G19 {NAME}: {}", run["ok"]);
    Ok(json!({
        "gate": "G19", "name": NAME, "program": program, "run": run, "clean_up": left, "forgot": forgot,
        "data_removed": removed.is_ok(), "data_error": removed.err(), "pass": pass,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_run_length_counts_every_quiet_case() {
        assert_eq!(
            QUIET, 7,
            "the password and unknown boxes, the aimed panel, the tray, the hung app, two Not nows"
        );
    }

    #[test]
    fn the_notes_name_each_cases_box_kind_and_the_arabic_lines_boxes() {
        let cases = [
            json!({ "case": "notepad_keys", "kind": "RichEditD2DPT / type 50030" }),
            json!({ "case": "arabic_0", "kind": "x", "placed": { "boxes": 2, "one_line": true } }),
        ];
        let n = notes(&cases);
        assert_eq!(n["kinds"]["notepad_keys"], "RichEditD2DPT / type 50030");
        assert_eq!(n["arabic_line"]["one_line"], true);
        assert!(n["screens"].as_u64().is_some_and(|s| s >= 1), "{n}");
    }
}
