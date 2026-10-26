//! G8's prediction part (`g8 predict`): idle private memory with predict on less with it off, set beside the owner's
//! figure as a reading, never a fail. The on run's typing churn may grow memory at most `[g8] churn_growth_mb`.
#![cfg(windows)]

use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::budget::{self, Part};
use crate::config::GatesConfig;
use crate::copyend::{self, Ended, Gone};
use crate::g8churn::{self, MB_DECIMALS};
use crate::g8start::Failed;
use crate::g8words::{self, Typed};
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
    let names = ["typing_working_mb", "typing_commit_mb"];
    g8churn::growth(names, t.before, t.after, limit)
}

/// True when both readings exist, the churn `held`, both copies `quit` by themselves and the owner's file came `back`.
pub(crate) fn passed(split: bool, held: bool, quit: [bool; RUNS], back: bool) -> bool {
    split && held && quit.iter().all(|&q| q) && back
}

/// One cold start: its idle memory, the typing churn when predict was on, and how the copy went.
struct Side {
    memory: Result<Memory, String>,
    typed: Option<Result<Typed, String>>,
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

/// The result line from both sides and the settings' put-back. It passes when both readings exist, the churn kept
/// within its limit, both copies quit by themselves and the owner's file came back; a churn that missed a word is
/// partial at best.
fn result(
    cfg: &GatesConfig,
    on: &Result<Side, Failed>,
    off: &Result<Side, Failed>,
    back: &Result<(), String>,
) -> Value {
    let (limit, owner_mb) = (cfg.g8.churn_growth_mb, cfg.g8_predict.owner_mb);
    let memory = |s: &Result<Side, Failed>| s.as_ref().ok().and_then(|s| s.memory.clone().ok());
    let split = memory(on).zip(memory(off)).map(|(a, b)| part(a, b));
    let typed = on.as_ref().ok().and_then(|s| s.typed.clone());
    let churned = typed.clone().and_then(Result::ok);
    let parts = churned.as_ref().map(|t| typing_parts(t, limit));
    let quit: [bool; RUNS] = [on, off].map(|s| s.as_ref().is_ok_and(|s| s.ended.ok()));
    let held = parts.as_ref().is_some_and(|p| budget::all_pass(p));
    let ok = passed(split.is_some(), held, quit, back.is_ok());
    json!({
        "gate": "G8", "part": PREDICT, "keyboard": product::NAME, "on": side_line(on), "off": side_line(off),
        "predict_mb": split, "owner_mb": owner_mb, "reading": split.map(|m| reading(m, owner_mb)),
        "typing": typed.as_ref().map(shown), "typing_parts": parts, "settings_put_back": back.is_ok(),
        "settings_error": back.as_ref().err(), "pass": out::pass(ok, churned.is_some_and(|t| t.whole())),
    })
}

/// Runs `g8 predict`, refused while the product runs: two cold starts of `[g8] program`, predict on then in `[kernel]
/// disabled`, each read after G8's warm-up, the on run then typing `[g8_predict]` words through chips. Both copies are
/// ended before the owner's file comes back byte for byte.
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
            note: None,
        }
    }

    fn ended(gone: Gone) -> Ended {
        Ended {
            gone,
            note: None,
            released: Vec::new(),
        }
    }

    /// A side read as `m`, with the churn `typed` when predict was on, whose copy went `gone`.
    fn side_of(m: Memory, typed: Option<Typed>, gone: Gone) -> Side {
        Side {
            memory: Ok(m),
            typed: typed.map(Ok),
            ended: ended(gone),
        }
    }

    #[test]
    fn the_result_passes_on_two_clean_quits_and_is_partial_after_a_missed_word() {
        let cfg = crate::config::load().expect("loads");
        let whole = typed(memory(60.0, 80.0), memory(60.5, 80.5));
        let off = Ok(side_of(memory(62.0, 87.0), None, Gone::Quit));
        let on = |t: &Typed| Ok(side_of(memory(64.0, 90.0), Some(t.clone()), Gone::Quit));
        let v = result(&cfg, &on(&whole), &off, &Ok(()));
        assert_eq!(v["pass"], true, "{v}");
        let mut missed = whole.clone();
        missed.picked -= 1;
        missed.note = Some("no chip offered the en prefix".into());
        let v = result(&cfg, &on(&missed), &off, &Ok(()));
        assert_eq!(v["pass"], out::PARTIAL, "{v}");
        assert_eq!(v["typing"]["note"], "no chip offered the en prefix");
        let v = result(&cfg, &on(&whole), &off, &Err("not back".into()));
        assert_eq!(v["pass"], false, "the owner's file did not come back");
    }

    #[test]
    fn an_off_side_never_started_after_a_stuck_copy_fails_the_result() {
        let cfg = crate::config::load().expect("loads");
        let whole = typed(memory(60.0, 80.0), memory(60.5, 80.5));
        let on = Ok(side_of(memory(64.0, 90.0), Some(whole), Gone::Running));
        let off = Err(Failed::from(STILL_RUNS.to_string()));
        let v = result(&cfg, &on, &off, &Ok(()));
        assert_eq!(v["pass"], false, "{v}");
        assert_eq!(v["off"]["error"], STILL_RUNS);
        assert!(v["predict_mb"].is_null(), "no off reading");
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
    fn a_copy_that_had_to_be_ended_fails_the_run() {
        assert!(passed(true, true, [true, true], true));
        assert!(
            !passed(true, true, [true, false], true),
            "the off copy was ended"
        );
        assert!(
            !passed(true, true, [true, true], false),
            "the owner's file did not come back"
        );
        assert!(!passed(false, true, [true, true], true), "no reading");
        assert!(!passed(true, false, [true, true], true), "the churn grew");
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
