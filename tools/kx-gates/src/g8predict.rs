//! G8's prediction part (`g8 predict`): two cold starts of `[g8] program`, with predict on and with it in `[kernel]
//! disabled`, each read for idle private memory after G8's warm-up. Their difference is the predict part, set beside
//! the owner's figure as a reading, never a fail. The on run then types `[g8_predict]` words through chips, over which
//! memory may grow at most `[g8] churn_growth_mb`. The owner's settings file is kept and put back byte for byte.
#![cfg(windows)]

use std::path::Path;
use std::time::Instant;

use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::budget::{self, Part};
use crate::g8churn::MB_DECIMALS;
use crate::g8start::{self, Started};
use crate::g8words::{self, Typed};
use crate::out::{self, say, shown};
use crate::ownset::{self, Kept};
use crate::proc::Memory;
use crate::{close, g8, product};

/// The name that runs G8's prediction part.
pub const PREDICT: &str = "predict";
/// Why the off run was not started.
const STILL_RUNS: &str =
    "the predict-on copy still runs, so no cold start was made with predict off; close it";

/// The predict part: memory with predict on less memory with it off.
pub(crate) fn part(on: Memory, off: Memory) -> Memory {
    Memory {
        working_mb: on.working_mb - off.working_mb,
        commit_mb: on.commit_mb - off.commit_mb,
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

/// The typing churn's growth of the working set and of the commit, each against `limit`, `[g8] churn_growth_mb`.
pub(crate) fn typing_parts(t: &Typed, limit: f64) -> [Part; 2] {
    let (b, a) = (t.before, t.after);
    [
        Part::at_most(
            "typing_working_mb",
            Some(a.working_mb - b.working_mb),
            limit,
        ),
        Part::at_most("typing_commit_mb", Some(a.commit_mb - b.commit_mb), limit),
    ]
}

/// One cold start: its idle memory, the typing churn when predict was on, and whether the copy quit.
struct Side {
    memory: Result<Memory, String>,
    typed: Option<Result<Typed, String>>,
    quit: bool,
}

/// Idle private memory once `[g8] warm_ms` passed since the keyboard showed.
fn idle(ctx: &Ctx, s: &Started) -> Result<Memory, String> {
    let shown = s.shown_at.ok_or("the keyboard did not show in time")?;
    std::thread::sleep(g8start::warm_left(
        shown,
        ctx.cfg.g8.warm_ms,
        Instant::now(),
    ));
    s.process.memory()
}

/// A cold start with the owner's settings set for predict `on`, measured, then asked to quit.
fn side(ctx: &Ctx, program: &Path, kept: &Kept, on: bool) -> Result<Side, String> {
    kept.write(&ownset::predict(&kept.table(), on)?)?;
    let s = g8start::start(ctx, program, &[], None)?;
    let memory = idle(ctx, &s);
    let typed = (on && memory.is_ok()).then(|| g8words::run(ctx, &s.process));
    let quit = close::quit_started(ctx, &s.process).unwrap_or(false);
    Ok(Side {
        memory,
        typed,
        quit,
    })
}

/// A side for the result line: its memory and whether it quit, or its error.
fn side_line(s: &Result<Side, String>) -> Value {
    match s {
        Ok(s) => json!({ "memory": shown(&s.memory), "quit": s.quit }),
        Err(e) => json!({ "error": e }),
    }
}

/// The result line from both sides and the settings' put-back. It passes when both readings exist, the churn kept
/// within its limit, both copies quit and the owner's file came back; a churn that missed a word is partial at best.
fn result(
    ctx: &Ctx,
    on: &Result<Side, String>,
    off: &Result<Side, String>,
    back: &Result<(), String>,
) -> Value {
    let (limit, owner_mb) = (ctx.cfg.g8.churn_growth_mb, ctx.cfg.g8_predict.owner_mb);
    let memory = |s: &Result<Side, String>| s.as_ref().ok().and_then(|s| s.memory.clone().ok());
    let split = memory(on).zip(memory(off)).map(|(a, b)| part(a, b));
    let typed = on.as_ref().ok().and_then(|s| s.typed.clone());
    let churned = typed.clone().and_then(Result::ok);
    let parts = churned.as_ref().map(|t| typing_parts(t, limit));
    let quit = [on, off].iter().all(|s| s.as_ref().is_ok_and(|s| s.quit));
    let held = parts.as_ref().is_some_and(|p| budget::all_pass(p));
    let ok = split.is_some() && held && quit && back.is_ok();
    json!({
        "gate": "G8", "part": PREDICT, "keyboard": product::NAME, "on": side_line(on), "off": side_line(off),
        "predict_mb": split, "owner_mb": owner_mb, "reading": split.map(|m| reading(m, owner_mb)),
        "typing": typed.as_ref().map(shown), "typing_parts": parts, "settings_put_back": back.is_ok(),
        "settings_error": back.as_ref().err(), "pass": out::pass(ok, churned.is_some_and(|t| t.whole())),
    })
}

/// Runs `g8 predict`, refused while the product runs, as both readings are cold starts.
fn predict(ctx: &Ctx) -> Result<Value, String> {
    g8::cold()?;
    let program = ctx.cfg.program(&ctx.cfg.g8.program);
    let kept = Kept::take(&product::files(&program)?)?;
    let on = side(ctx, &program, &kept, true);
    let off = match &on {
        Ok(s) if !s.quit => Err(STILL_RUNS.to_string()),
        _ => side(ctx, &program, &kept, false),
    };
    let back = kept.put_back();
    let v = result(ctx, &on, &off, &back);
    say!("G8 {PREDICT}: {}", v["reading"]);
    Ok(v)
}

/// Runs `g8 predict`; none for another name.
pub fn run(ctx: &Ctx, name: &str) -> Option<Result<Value, String>> {
    (name == PREDICT).then(|| predict(ctx))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory(working_mb: f64, commit_mb: f64) -> Memory {
        Memory {
            working_mb,
            commit_mb,
        }
    }

    fn typed(before: Memory, after: Memory) -> Typed {
        Typed {
            words: 30,
            picked: 30,
            before,
            after,
        }
    }

    #[test]
    fn the_predict_part_is_the_on_reading_less_the_off_reading() {
        let got = part(memory(64.5, 90.0), memory(62.5, 87.0));
        assert_eq!(got, memory(2.0, 3.0));
        let line = reading(part(memory(64.5, 90.0), memory(62.3, 87.0)), 2.0);
        assert!(line.contains("2.2 MB working set"), "{line}");
        assert!(
            line.contains("3.0 MB commit") && line.contains("about 2.0 MB"),
            "{line}"
        );
    }

    #[test]
    fn a_predict_part_below_nothing_still_reads_as_a_number() {
        let line = reading(part(memory(60.0, 80.0), memory(60.5, 80.0)), 2.0);
        assert!(line.contains("-0.5 MB working set"), "{line}");
    }

    #[test]
    fn the_typing_churn_passes_within_the_growth_limit_and_fails_past_it() {
        let within = typing_parts(&typed(memory(60.0, 80.0), memory(62.0, 82.5)), 3.0);
        assert!(budget::all_pass(&within));
        assert_eq!(within.map(|p| p.got), [Some(2.0), Some(2.5)]);
        let past = typing_parts(&typed(memory(60.0, 80.0), memory(63.5, 81.0)), 3.0);
        assert_eq!(
            past.map(|p| p.pass),
            [false, true],
            "the working set grew past it"
        );
        let commit = typing_parts(&typed(memory(60.0, 80.0), memory(60.0, 83.1)), 3.0);
        assert_eq!(
            commit.map(|p| p.pass),
            [true, false],
            "the commit grew past it"
        );
    }
}
