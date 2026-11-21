//! Compare's cloud models (edge 12): the recording goes once to each ticked cloud model before any local model writes
//! it, each one's text or fault is its own result, the run ends once every model ended, and Cancel drops the uploads.

use kx_module_api::ports::{CloudFault, CloudNote};

use super::*;

/// A compare of `models` and `clouds` that recorded speech and stopped, with its run's id.
fn recorded_with(models: Vec<Entrant>, clouds: Vec<Remote>) -> (Compare, u32) {
    let mut compare = Compare::default();
    let take = record(&mut compare, models, clouds);
    (compare, take)
}

/// Starts `compare` on `models` and `clouds`, hears speech and stops; gives the run's id.
fn record(compare: &mut Compare, models: Vec<Entrant>, clouds: Vec<Remote>) -> u32 {
    let plan = Plan {
        clouds,
        ..plan(SET, models)
    };
    let take = match compare.start(plan, &settings())[..] {
        [Act::Open { take, .. }] => take,
        ref acts => panic!("no open: {acts:?}"),
    };
    compare.note(take, SpeechNote::Heard);
    compare.stop();
    take
}

/// Each model's step, local ones first.
fn steps(compare: &Compare) -> Vec<RunStep> {
    let (_, runs) = compare.running().unwrap();
    runs.iter().map(|(_, step)| *step).collect()
}

#[test]
fn the_recording_goes_to_each_ticked_cloud_model_before_any_local_model_writes() {
    let (mut compare, take) = recorded_with(vec![entrant(1, false)], vec![remote(1), remote(2)]);
    let acts = compare.note(take, done(0, 0));
    let [
        Act::Send {
            take: sent,
            lang,
            to,
        },
        Act::Run {
            call: RunCall::Again(_),
            ..
        },
    ] = &acts[..]
    else {
        panic!("not sent first: {acts:?}");
    };
    assert_eq!(
        (*sent, lang.as_str(), to.clone()),
        (take, "en", vec![remote(1), remote(2)])
    );
    let writing = [RunStep::Writing; 3];
    assert_eq!(steps(&compare), writing);
}

#[test]
fn a_cloud_fault_fails_that_model_alone_and_the_run_ends_once_every_model_ended() {
    let (mut compare, take) = recorded_with(vec![entrant(1, false)], vec![remote(1), remote(2)]);
    compare.note(take, done(0, 0));
    compare.note(take, piece(SET));
    let local_done = compare.note(take, done(1_000, 800));
    assert!(
        local_done.is_empty() && compare.results().is_empty(),
        "the cloud models still write"
    );
    assert!(
        compare
            .net(take, 0, CloudNote::Fault(CloudFault::Key), 0, None)
            .is_empty()
    );
    let text = secret(SET);
    compare.net(take, 1, CloudNote::Piece { index: 0, text }, 0, None);
    let wait = RunStep::Done { wait_ms: 1_000 };
    assert_eq!(steps(&compare), [wait, RunStep::Failed, RunStep::Writing]);
    let last = compare.net(take, 1, CloudNote::Done, 1_500, None);
    let saved = last.iter().find_map(|a| match a {
        Act::Save(board) => Some(board.scores.len()),
        _ => None,
    });
    assert_eq!(saved, Some(1), "a cloud model's numbers are not kept");
    let lines = compare.results();
    let cloud = &lines[1];
    assert_eq!(lines.len(), 2, "the failed one has no result");
    assert_eq!((cloud.cloud.clone(), cloud.print), (Some(remote(2)), None));
    assert_eq!((cloud.wrong, cloud.wait_ms, cloud.mb), (Some(0), 1_500, 0));
}

#[test]
fn a_cloud_model_that_failed_keeps_its_reason_for_the_page() {
    let (mut compare, take) = recorded_with(vec![entrant(1, false)], vec![remote(1), remote(2)]);
    compare.note(take, done(0, 0));
    assert_eq!(compare.why(1), None, "none before a fault");
    compare.net(take, 0, CloudNote::Fault(CloudFault::Key), 0, None);
    assert_eq!(steps(&compare)[1], RunStep::Failed);
    assert_eq!(
        (compare.why(0), compare.why(1), compare.why(2)),
        (None, Some((CloudFault::Key, None)), None),
        "only the failed cloud model, after the local one"
    );
    compare.cancel();
    assert_eq!(compare.why(1), None, "a cancelled run shows none");
    let take = record(
        &mut compare,
        vec![entrant(1, false)],
        vec![remote(1), remote(2)],
    );
    compare.note(take, done(0, 0));
    assert_eq!(compare.why(1), None, "a new run forgets it");
}

#[test]
fn cancel_drops_the_uploads_and_a_later_note_is_not_heard() {
    let (mut compare, take) = recorded_with(Vec::new(), vec![remote(1)]);
    let acts = compare.note(take, done(0, 0));
    assert!(
        matches!(&acts[..], [Act::Send { .. }]),
        "no local model, so no Again"
    );
    let cancel = compare.cancel();
    assert!(cancel.contains(&Act::Halt(take)));
    assert!(compare.net(take, 0, CloudNote::Done, 0, None).is_empty());
    assert!(compare.results().is_empty());
}

#[test]
fn nothing_goes_to_a_cloud_model_before_the_recording_ended() {
    let mut compare = Compare::default();
    let plan = Plan {
        clouds: vec![remote(1)],
        ..plan(SET, Vec::new())
    };
    let acts = compare.start(plan, &settings());
    assert!(
        matches!(&acts[..], [Act::Open { .. }]),
        "a cloud-only run still records"
    );
    assert!(!acts.iter().any(|a| matches!(a, Act::Send { .. })));
    assert!(
        compare
            .stop()
            .iter()
            .all(|a| !matches!(a, Act::Send { .. })),
        "no speech heard yet"
    );
}
