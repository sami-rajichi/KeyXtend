//! G19's plan on the product: the parts it judges, in order, and the verdicts over each selection case it watched; the
//! notes are recorded and never fail a run.
#![cfg(windows)]

use serde_json::{Value, json};

/// Each case's shown keycaps against the box's own.
pub const CHIPS: &str = "chips";
/// Each case's strip showed with the clipboard untouched.
pub const SEEN: &str = "seen";
/// Each case kept the window in front.
pub const FOCUS: &str = "focus";
/// Each placed case sat where `anchor` says.
pub const PLACE: &str = "place";
/// The groups judged over the cases, in order.
pub const GROUPS: [&str; 4] = [SEEN, FOCUS, PLACE, CHIPS];
/// The terminals step, by its key in the run; `g19 terminals` runs it alone.
pub const TERMINALS: &str = "terminals";
/// The parts each run step gives, in order.
pub const STEPS: [&str; 8] = [
    "edits",
    "caret",
    "hides",
    "ours",
    "near_bottom",
    TERMINALS,
    "hung",
    "translate",
];
/// What is recorded beside them, never judged.
pub const NOTES: &str = "notes";
/// A place case's mark of whether it ran: one that did not is never a pass.
pub const EXERCISED: &str = "exercised";
/// The place cases, by their key in the run, that may not run when their window cannot move.
pub const PLACES: [&str; 2] = ["near_bottom", "second"];
/// The parts of a step, by step and key, that may not run: the console when its window never shows.
pub const PARTS: [(&str, &str); 1] = [(TERMINALS, "console")];

/// One group's verdict over `cases`: every case that holds `key` holds it true, and at least one does.
fn group(cases: &[Value], key: &str) -> Value {
    let told: Vec<&Value> = cases.iter().filter_map(|c| c.get(key)).collect();
    let ok = !told.is_empty() && told.iter().all(|v| **v == true);
    json!({ "ok": ok, "cases": told.len() })
}

/// The judged groups over every watched case.
pub fn groups(cases: &[Value]) -> Value {
    let all = GROUPS.iter().map(|k| ((*k).to_string(), group(cases, k)));
    Value::Object(all.collect())
}

/// True when place case `key` of `run` says it did not run.
fn unrun(run: &Value, key: &str) -> bool {
    PLACES.contains(&key) && run[key][EXERCISED] == false
}

/// The parts of `run` that did not run, by name, which go to the manual check.
pub fn unrun_parts(run: &Value) -> Vec<&'static str> {
    let parts = PARTS.into_iter();
    let unrun = parts.filter(|(s, k)| run[s][k][EXERCISED] == false);
    unrun.map(|(_, k)| k).collect()
}

/// The place cases of `run` that did not run or ended in an error, then the parts that did not run, by name, which go
/// to the manual check.
pub fn unexercised(run: &Value) -> Vec<&'static str> {
    let places = PLACES.into_iter().filter(|k| run[k][EXERCISED] != true);
    places.chain(unrun_parts(run)).collect()
}

/// True when every group and every step in `run` that ran passed; a missing one fails, and a place case that did not
/// run and the notes count for nothing.
pub fn passed(run: &Value) -> bool {
    let grouped = GROUPS.iter().all(|k| run["cases"][k]["ok"] == true);
    grouped && STEPS.iter().all(|s| run[s]["ok"] == true || unrun(run, s))
}

/// True when `step`, run alone, passed, and so did each group its cases tell; a group none tells counts for nothing,
/// but one must.
pub fn step_passed(run: &Value, step: &str) -> bool {
    let told: Vec<&str> = (GROUPS.into_iter())
        .filter(|k| run["cases"][k]["cases"] != 0)
        .collect();
    let grouped = !told.is_empty() && told.iter().all(|k| run["cases"][k]["ok"] == true);
    grouped && run[step]["ok"] == true
}

/// True when a timer that went after `after_ms` kept `due_ms` within `slack_ms` either way.
pub fn on_time(after_ms: u64, due_ms: u64, slack_ms: u64) -> bool {
    (due_ms.saturating_sub(slack_ms)..=due_ms.saturating_add(slack_ms)).contains(&after_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A case that passed all four checks.
    fn good() -> Value {
        json!({ "seen": true, "focus": true, "place": true, "chips": true })
    }

    /// A run whose every group and step passed.
    fn run() -> Value {
        let mut v = json!({ "notes": { "ok": false } });
        v["cases"] = groups(&[good(), good()]);
        for step in STEPS {
            v[step] = json!({ "ok": true });
        }
        v
    }

    #[test]
    fn a_group_passes_when_every_case_that_tells_it_passed() {
        let mut quiet = good();
        quiet.as_object_mut().expect("an object").remove(PLACE);
        let g = groups(&[good(), quiet]);
        for key in GROUPS {
            assert_eq!(g[key]["ok"], true, "{key}");
        }
        assert_eq!(g[PLACE]["cases"], 1, "the quiet case places nothing");
        let mut bad = good();
        bad[FOCUS] = json!(false);
        assert_eq!(groups(&[good(), bad])[FOCUS]["ok"], false);
        assert_eq!(groups(&[])[SEEN]["ok"], false, "nothing watched");
    }

    #[test]
    fn the_run_passes_when_every_group_and_step_did_and_notes_never_count() {
        assert!(passed(&run()));
        for step in STEPS {
            let mut v = run();
            v[step] = json!({ "ok": false, "error": "no strip" });
            assert!(!passed(&v), "{step}");
            v.as_object_mut().expect("an object").remove(step);
            assert!(!passed(&v), "{step} missing");
        }
        let mut v = run();
        v["cases"][CHIPS]["ok"] = json!(false);
        assert!(!passed(&v));
    }

    #[test]
    fn a_place_case_that_did_not_run_is_listed_and_never_a_pass() {
        let mut v = run();
        v["near_bottom"] = json!({ EXERCISED: false, "reason": "not moved" });
        v["second"] = json!({ EXERCISED: false, "reason": "one screen" });
        assert_eq!(unexercised(&v), ["near_bottom", "second"]);
        assert!(passed(&v), "the cases that ran passed");
        v["near_bottom"] = json!({ EXERCISED: true, "ok": false });
        assert!(!passed(&v) && unexercised(&v) == ["second"]);
        v["near_bottom"] = json!({ EXERCISED: true, "ok": true });
        assert!(passed(&v));
        v["caret"] = json!({ EXERCISED: false });
        assert!(!passed(&v), "only a place case may skip");
        let mut v = run();
        v["near_bottom"] = json!({ "ok": false, "error": "no rows" });
        v["second"] = json!({ "screens": 2, "error": "not in front" });
        assert_eq!(
            unexercised(&v),
            PLACES,
            "an errored case goes to the manual check"
        );
        assert!(!passed(&v), "and still fails");
        v["near_bottom"] = json!({ EXERCISED: true, "ok": true });
        v["second"] = json!({ EXERCISED: true, "screens": 2 });
        assert!(unexercised(&v).is_empty() && passed(&v));
    }

    #[test]
    fn a_console_that_did_not_run_goes_to_the_manual_check() {
        let mut v = run();
        v["near_bottom"] = json!({ EXERCISED: true, "ok": true });
        v["second"] = json!({ EXERCISED: true });
        v["terminals"] =
            json!({ "console": { EXERCISED: false, "reason": "no window" }, "ok": true });
        assert_eq!(unexercised(&v), ["console"]);
        assert!(passed(&v), "the terminals step judged it already");
        v["terminals"]["console"] = json!({ "ok": true });
        assert!(unexercised(&v).is_empty(), "a console that ran");
        let alone = json!({ TERMINALS: { "console": { EXERCISED: false } } });
        assert_eq!(
            unrun_parts(&alone),
            ["console"],
            "no place case in a run alone"
        );
    }

    #[test]
    fn a_step_run_alone_passes_with_the_groups_its_cases_tell() {
        let quiet = json!({ "seen": true, "focus": true, "chips": true });
        let mut v = json!({ TERMINALS: { "ok": true } });
        v["cases"] = groups(std::slice::from_ref(&quiet));
        assert!(
            step_passed(&v, TERMINALS),
            "no case placed, so place counts for nothing"
        );
        v["cases"] = groups(&[
            quiet,
            json!({ "seen": false, "focus": true, "chips": false }),
        ]);
        assert!(!step_passed(&v, TERMINALS));
        v["cases"] = groups(&[]);
        assert!(!step_passed(&v, TERMINALS), "no case at all");
        v["cases"] = groups(&[good()]);
        v[TERMINALS]["ok"] = json!(false);
        assert!(!step_passed(&v, TERMINALS));
    }

    #[test]
    fn a_timer_passes_within_its_slack_either_way() {
        assert!(on_time(5_000, 5_000, 1_000));
        assert!(on_time(4_000, 5_000, 1_000) && on_time(6_000, 5_000, 1_000));
        assert!(!on_time(3_999, 5_000, 1_000) && !on_time(6_001, 5_000, 1_000));
        assert!(!on_time(0, 5_000, 1_000), "gone at once");
    }
}
