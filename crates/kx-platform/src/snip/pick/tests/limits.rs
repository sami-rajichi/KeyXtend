//! The ask's limits: the job's memory cap and the dialog's wait come from them, the same limits keep the worker, new
//! ones start a new worker after the old one's goodbye, and limits out of range start nothing.

use super::*;

/// The user pressed Cancel.
fn cancel(id: u32) -> PickReply<'static> {
    PickReply::Cancelled { id }
}

#[test]
fn the_job_runs_as_its_limits_say_and_new_limits_start_a_new_worker() {
    let script = Script::default();
    let asker = asker(&script);
    for _ in 0..2 {
        script.act(PickAct::Answer(cancel));
        assert!(matches!(asker.save(&proposed(), &limits()), Ok(None)));
    }
    assert_eq!(script.spawns(), 1, "the same limits keep the worker");
    let longer = PickLimits {
        job_mb: limits().job_mb + 1,
        dialog_min: limits().dialog_min + 1,
        ..limits()
    };
    script.act(PickAct::Answer(cancel));
    assert_eq!(asker.folder(&pictures(), &longer), Ok(None));
    let launches = [Launch::pick(limits().job_mb), Launch::pick(longer.job_mb)];
    assert_eq!(script.launches(), launches);
    let calls = script.calls();
    let bye = calls.iter().position(|c| *c == Call::Bye);
    let late = Call::Reply {
        within: dialog(&longer),
    };
    let new = calls.iter().position(|c| *c == late);
    assert!(bye.is_some() && bye < new, "goodbye first: {calls:?}");
}

#[test]
fn limits_out_of_range_start_no_worker() {
    let script = Script::default();
    let asker = asker(&script);
    let bad = PickLimits {
        dialog_min: limits().dialog_min - 1,
        ..limits()
    };
    let saved = asker.save(&proposed(), &bad).err();
    assert_eq!(saved, Some(AskError::Range("dialog_min")));
    let order = PickLimits {
        restart_ms: limits().restart_max_ms + 1,
        ..limits()
    };
    let folder = asker.folder(&pictures(), &order).err();
    assert_eq!(folder, Some(AskError::Range("restart_ms")));
    assert_eq!(script.tries(), 0);
}
