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
/// The parts each run step gives, in order.
pub const STEPS: [&str; 8] = [
    "edits",
    "caret",
    "hides",
    "ours",
    "near_bottom",
    "terminals",
    "hung",
    "translate",
];
/// What is recorded beside them, never judged.
pub const NOTES: &str = "notes";
/// A place case's mark of whether it ran: one that did not is never a pass.
pub const EXERCISED: &str = "exercised";
/// The place cases, by their key in the run, that may not run when their window cannot move.
pub const PLACES: [&str; 2] = ["near_bottom", "second"];

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

/// The place cases of `run` that did not run, by name, which go to the manual check.
pub fn unexercised(run: &Value) -> Vec<&'static str> {
    PLACES.into_iter().filter(|k| unrun(run, k)).collect()
}

/// True when every group and every step in `run` that ran passed; a missing one fails, and a place case that did not
/// run and the notes count for nothing.
pub fn passed(run: &Value) -> bool {
    let grouped = GROUPS.iter().all(|k| run["cases"][k]["ok"] == true);
    grouped && STEPS.iter().all(|s| run[s]["ok"] == true || unrun(run, s))
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
        assert!(unexercised(&run()).is_empty());
    }

    #[test]
    fn a_timer_passes_within_its_slack_either_way() {
        assert!(on_time(5_000, 5_000, 1_000));
        assert!(on_time(4_000, 5_000, 1_000) && on_time(6_000, 5_000, 1_000));
        assert!(!on_time(3_999, 5_000, 1_000) && !on_time(6_001, 5_000, 1_000));
        assert!(!on_time(0, 5_000, 1_000), "gone at once");
    }
}
