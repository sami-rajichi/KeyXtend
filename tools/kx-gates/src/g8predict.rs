//! G8's prediction part (`g8 predict`): idle private memory with predict on less with it off, set beside the owner's
//! figure as a reading, never a fail. The on run's one-language typing churn may grow memory at most `[g8]
//! churn_growth_mb`; its mixed-language churn's growth is a reading, never a fail (owner, 2026-10-26).
#![cfg(windows)]

use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::budget::{self, Part};
use crate::config::GatesConfig;
use crate::copyend::{self, Ended, Gone};
use crate::g8churn::{self, MB_DECIMALS};
use crate::g8start::Failed;
use crate::g8words::{self, Churns, Typed};
use crate::out::{self, say, shown};
use crate::owncopy::Session;
use crate::proc::Memory;
use crate::product;

/// The name that runs G8's prediction part.
pub const PREDICT: &str = "predict";
/// The cold starts `g8 predict` makes: predict on, then off.
pub const RUNS: usize = 2;
/// Why the off run was not started.
const STILL_RUNS: &str =
    "the predict-on copy could not be ended, so no cold start was made with predict off; close it";

/// Memory `a` less memory `b`: the predict part (on less off), or a churn's growth (after less before).
pub(crate) fn less(a: Memory, b: Memory) -> Memory {
    Memory {
        working_mb: a.working_mb - b.working_mb,
        commit_mb: a.commit_mb - b.commit_mb,
    }
}

/// The reading line: the predict part beside the owner's figure, in MB.
pub(crate) fn reading(part: Memory, owner_mb: f64) -> String {
    let (w, c) = (part.working_mb, part.commit_mb);
    format!(
        "predict part {w:.MB_DECIMALS$} MB working set, {c:.MB_DECIMALS$} MB commit; the owner's figure is about \
         {owner_mb:.MB_DECIMALS$} MB"
    )
}

/// The mixed churn's line: its growth in MB over the words it picked; a reading, never a fail.
pub(crate) fn mixed_reading(t: &Typed) -> String {
    let grown = less(t.after, t.before);
    let (w, c, picked, words) = (grown.working_mb, grown.commit_mb, t.picked, t.words);
    format!(
        "mixed typing grew {w:.MB_DECIMALS$} MB working set, {c:.MB_DECIMALS$} MB commit over {picked} of {words} \
         words; a reading, never a fail"
    )
}

/// The typing churn's growth of the working set and of the commit, each against `limit`, `[g8] churn_growth_mb`.
pub(crate) fn typing_parts(t: &Typed, limit: f64) -> [Part; 2] {
    let names = ["typing_working_mb", "typing_commit_mb"];
    g8churn::growth(names, t.before, t.after, limit)
}

/// True when both readings exist, the churn `held`, both copies `quit` by themselves and the owner's file came `back`.
pub(crate) fn passed(split: bool, held: bool, quit: [bool; RUNS], back: bool) -> bool {
    split && held && quit.iter().all(|&q| q) && back
}

/// One cold start: its idle memory, the typing churns when predict was on, and how the copy went.
struct Side {
    memory: Result<Memory, String>,
    typed: Option<Result<Churns, String>>,
    ended: Ended,
}

/// A cold start on the gate's settings with predict `on`, read once `[g8] warm_ms` passed, then ended.
fn side(ctx: &Ctx, session: &Session, on: bool) -> Result<Side, Failed> {
    let copy = session.start(on, &[])?;
    let ready = copy.ready(ctx.cfg.g8.warm_ms);
    let memory = ready.clone().and_then(|s| s.process.memory());
    let typed = match ready {
        Ok(s) if on && memory.is_ok() => Some(g8words::run(ctx, &s.process)),
        _ => None,
    };
    Ok(Side {
        memory,
        typed,
        ended: copy.end(),
    })
}

/// A side for the result line: its memory and how its copy went, or its error and how a copy it started went.
fn side_line(s: &Result<Side, Failed>) -> Value {
    match s {
        Ok(s) => json!({ "memory": shown(&s.memory), "copy": s.ended.line() }),
        Err(f) => json!({ "error": f.why, "copy": f.ended.as_ref().map(Ended::line) }),
    }
}

/// How the copy a side started went; none when none started.
fn gone(s: &Result<Side, Failed>) -> Option<Gone> {
    match s {
        Ok(s) => Some(s.ended.gone),
        Err(f) => f.ended.as_ref().map(|e| e.gone),
    }
}

/// A churn for the result line, or why it gave none; none when it never ran.
type Shown = Option<Result<Typed, String>>;

/// The on side's judged churn and its mixed one; a failed typing run is the judged churn's error alone.
fn typing(on: &Result<Side, Failed>) -> (Shown, Shown) {
    let churns = on.as_ref().ok().and_then(|s| s.typed.clone());
    let mixed = churns.clone().and_then(Result::ok).map(|c| c.mixed);
    (churns.map(|r| r.map(|c| c.judged)), mixed)
}

/// The result line from both sides and the settings' put-back. It passes when both readings exist, the judged churn
/// kept within its limit, both copies quit by themselves and the owner's file came back; a judged churn that missed a
/// word is partial at best. The mixed churn only reports.
fn result(
    cfg: &GatesConfig,
    on: &Result<Side, Failed>,
    off: &Result<Side, Failed>,
    back: &Result<(), String>,
) -> Value {
    let (limit, owner_mb) = (cfg.g8.churn_growth_mb, cfg.g8_predict.owner_mb);
    let memory = |s: &Result<Side, Failed>| s.as_ref().ok().and_then(|s| s.memory.clone().ok());
    let split = memory(on).zip(memory(off)).map(|(a, b)| less(a, b));
    let (typed, mixed) = typing(on);
    let churned = typed.clone().and_then(Result::ok);
    let parts = churned.as_ref().map(|t| typing_parts(t, limit));
    let quit: [bool; RUNS] = [on, off].map(|s| s.as_ref().is_ok_and(|s| s.ended.ok()));
    let held = parts.as_ref().is_some_and(|p| budget::all_pass(p));
    let ok = passed(split.is_some(), held, quit, back.is_ok());
    let mixed_line = mixed
        .as_ref()
        .and_then(|m| m.as_ref().ok())
        .map(mixed_reading);
    json!({
        "gate": "G8", "part": PREDICT, "keyboard": product::NAME, "on": side_line(on), "off": side_line(off),
        "predict_mb": split, "owner_mb": owner_mb, "reading": split.map(|m| reading(m, owner_mb)),
        "typing": typed.as_ref().map(shown), "typing_parts": parts, "typing_mixed": mixed.as_ref().map(shown),
        "mixed_reading": mixed_line, "settings_put_back": back.is_ok(), "settings_error": back.as_ref().err(),
        "pass": out::pass(ok, churned.is_some_and(|t| t.whole())),
    })
}

/// Runs `g8 predict`, refused while the product runs: two cold starts of `[g8] program`, predict on then in `[kernel]
/// disabled`, each read after G8's warm-up, the on run then typing `[g8_predict]` words through chips, one language
/// then all in turn. Both copies are ended before the owner's file comes back byte for byte.
fn predict(ctx: &Ctx) -> Result<Value, String> {
    let session = Session::open(ctx)?;
    let on = side(ctx, &session, true);
    let off = if copyend::may_follow(gone(&on)) {
        side(ctx, &session, false)
    } else {
        Err(Failed::from(STILL_RUNS.to_string()))
    };
    let back = session.close();
    let v = result(ctx.cfg, &on, &off, &back);
    say!("G8 {PREDICT}: {}; {}", v["reading"], v["mixed_reading"]);
    Ok(v)
}

/// Runs `g8 predict`; none for another name.
pub fn run(ctx: &Ctx, name: &str) -> Option<Result<Value, String>> {
    (name == PREDICT).then(|| predict(ctx))
}

#[cfg(test)]
mod tests;
