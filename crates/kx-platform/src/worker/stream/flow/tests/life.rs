//! A stream worker's life: a quit replaced at once, even before its close is heard, and a death held back; asks on
//! lost or unknown runs, asks with another id and notes on ids never asked refused; a watch dropped by an ask with no
//! wait, idle and stopped workers saying goodbye, a reader or a start that fails, and an ended worker's frames dropped.

use super::*;

#[test]
fn a_worker_that_closes_with_nothing_watched_is_replaced_with_no_backoff() {
    let mut rig = rig();
    assert_eq!(rig.flow.wake_in(), None, "no worker yet");
    rig.script.act(VoiceAct::Say(vec![devices]));
    rig.begin(list).unwrap();
    rig.heard();
    rig.script.quit(DIED);
    assert_eq!(rig.heard(), [Told::Lost(Lost::Gone)]);
    let within = ms(TIMES.deadline_ms);
    let calls = rig.script.calls();
    assert!(calls.ends_with(&[Call::Code { within }, Call::End]));
    assert_eq!(rig.begin(list), Ok(2), "started again at once");
    assert_eq!(rig.script.spawns(), 2);
}

#[test]
fn a_fresh_ask_replaces_a_worker_that_quit_before_its_close_was_heard() {
    let mut rig = rig();
    rig.script.act(VoiceAct::Say(vec![devices]));
    rig.begin(list).unwrap();
    rig.heard();
    rig.script.quit(DIED);
    assert_eq!(
        rig.begin(list),
        Ok(2),
        "nothing watched: started again at once"
    );
    assert_eq!(rig.script.spawns(), 2);
    assert_eq!(
        rig.heard(),
        [Told::Lost(Lost::Gone)],
        "its close is dropped"
    );
    let quit = Call::Code {
        within: Duration::ZERO,
    };
    assert!(rig.script.calls().contains(&quit), "asked without a wait");
    rig.script.quit(DIED);
    assert_eq!(rig.begin(list), down(ms(TIMES.restart_ms)), "a watched one");
}

#[test]
fn a_worker_that_closes_while_watched_is_held_back() {
    let mut rig = rig();
    rig.script.act(VoiceAct::Say(vec![open]));
    rig.begin(start).unwrap();
    rig.heard();
    rig.script.quit(DIED);
    assert_eq!(rig.heard(), [Told::Lost(Lost::Gone)]);
    assert_eq!(rig.begin(start), down(ms(TIMES.restart_ms)));
}

#[test]
fn an_ask_on_a_run_goes_to_its_worker_and_one_on_a_lost_or_unknown_run_is_gone() {
    let mut rig = rig();
    let gone = Err(Unsent::Lost(Lost::Gone));
    assert_eq!(rig.begin(start), Ok(1));
    assert_eq!(rig.flow.ask(stop(1), Wait::Each(WAIT)), Ok(()));
    assert_eq!(rig.flow.ask(stop(2), Wait::Each(WAIT)), gone, "never begun");
    let quit = Call::Code {
        within: Duration::ZERO,
    };
    let asked = [
        Call::Voice(start(1)),
        quit.clone(),
        Call::Voice(stop(1)),
        quit,
    ];
    assert_eq!(rig.script.calls(), asked, "a quit checked first");
    assert_eq!(rig.after(WAIT), [Told::Lost(Lost::Late)]);
    assert_eq!(
        rig.flow.ask(stop(1), Wait::Each(WAIT)),
        gone,
        "its worker is gone"
    );
    assert_eq!(rig.script.spawns(), 1, "and none started for it");
    rig.clock.advance(ms(TIMES.restart_ms));
    assert_eq!(rig.begin(start), Ok(2));
    assert_eq!(
        rig.flow.ask(stop(1), Wait::Each(WAIT)),
        gone,
        "run 1 is not here"
    );
    assert_eq!(rig.flow.ask(stop(2), Wait::Each(WAIT)), Ok(()));
}

#[test]
fn an_ask_the_frames_refuse_is_never_sent_and_starts_nothing() {
    let mut rig = rig();
    let refused = Unsent::Refused(IpcError::BadField);
    assert_eq!(rig.begin(bad_start), Err(refused));
    assert_eq!(rig.script.tries(), 0);
    assert_eq!(rig.begin(start), Ok(1), "the refused ask took no id");
    assert_eq!(rig.flow.ask(bad_start(1), Wait::Each(WAIT)), Err(refused));
    assert_eq!(rig.script.calls(), [Call::Voice(start(1))]);
}

#[test]
fn a_fresh_ask_built_with_another_id_than_given_is_refused() {
    let mut rig = rig();
    let refused = Err(Unsent::Refused(IpcError::BadField));
    assert_eq!(rig.flow.begin(|id| list(id + 1), Wait::Off), refused);
    assert_eq!(rig.script.tries(), 0, "no worker started");
    assert_eq!(rig.begin(list), Ok(1), "and it took no id");
}

#[test]
fn a_note_on_an_id_its_worker_was_never_asked_ends_it() {
    for (stray, why) in [(1, "a lost run's id"), (3, "an id never handed out")] {
        let mut rig = rig();
        rig.begin(start).unwrap();
        rig.after(WAIT);
        rig.clock.advance(ms(TIMES.restart_ms));
        assert_eq!(rig.begin(start), Ok(2));
        rig.script.say(Ok(level(stray)));
        rig.script.say(Ok(level(2)));
        assert_eq!(rig.heard(), [Told::Lost(Lost::Gone)], "{why}");
        assert_eq!(rig.script.calls().last(), Some(&Call::End));
        let again = ms(TIMES.restart_ms) * 2;
        assert_eq!(rig.begin(start), down(again), "held back, {why}");
    }
}

#[test]
fn an_ask_with_no_wait_ends_its_watch() {
    let mut rig = rig();
    rig.script.act(VoiceAct::Say(vec![open]));
    rig.begin(start).unwrap();
    rig.heard();
    rig.flow.ask(close(1), Wait::Off).unwrap();
    assert_eq!(rig.after(WAIT * 2), [], "nothing is watched");
    assert_eq!(rig.script.calls().last(), Some(&Call::Voice(close(1))));
}

#[test]
fn an_idle_worker_says_goodbye_and_is_told_idle_but_a_watched_one_stays() {
    let mut rig = rig();
    rig.script.act(VoiceAct::Say(vec![devices]));
    rig.begin(list).unwrap();
    rig.heard();
    assert_eq!(rig.after(idle().saturating_sub(TICK)), []);
    assert_eq!(rig.after(TICK), [Told::Idle]);
    let within = ms(TIMES.deadline_ms);
    let calls = rig.script.calls();
    assert!(calls.ends_with(&[Call::Bye, Call::Code { within }, Call::End]));
    assert_eq!(rig.flow.wake_in(), None, "no worker");
    assert_eq!(rig.begin(list), Ok(2), "a fresh ask starts another at once");
    let mut rig = super::rig();
    rig.flow.begin(start, Wait::Each(idle() * 2)).unwrap();
    assert_eq!(rig.after(idle()), [], "still watched");
}

#[test]
fn a_stopped_or_dropped_flow_says_goodbye_and_tells_nothing() {
    let mut rig = rig();
    let within = ms(TIMES.deadline_ms);
    let bye = [Call::Bye, Call::Code { within }, Call::End];
    rig.begin(start).unwrap();
    rig.flow.stop();
    assert!(rig.script.calls().ends_with(&bye));
    assert_eq!(rig.heard(), []);
    assert_eq!(
        rig.flow.ask(stop(1), Wait::Off),
        Err(Unsent::Lost(Lost::Gone))
    );
    assert_eq!(rig.begin(start), Ok(2), "no backoff after a stop");
    let Rig { flow, script, .. } = rig;
    drop(flow);
    let mut want = vec![Call::Voice(start(1))];
    want.extend(bye.clone());
    want.push(Call::Voice(start(2)));
    want.extend(bye);
    assert_eq!(script.calls(), want);
}

#[test]
fn a_reader_or_a_start_that_fails_holds_the_next_start_back() {
    let mut rig = rig_with(Err(SpawnError::Thread));
    let first = ms(TIMES.restart_ms);
    assert_eq!(rig.begin(list), down(first));
    assert_eq!(rig.script.calls(), [Call::End], "the worker was ended");
    let mut rig = super::rig();
    rig.script.refuse(1);
    assert_eq!(rig.begin(list), down(first));
    assert_eq!((rig.script.tries(), rig.script.spawns()), (1, 0));
}

#[test]
fn a_frame_of_an_ended_worker_is_dropped() {
    let mut rig = rig();
    rig.begin(start).unwrap();
    rig.heard();
    let (old, _) = rig.live[0];
    assert_eq!(rig.after(WAIT), [Told::Lost(Lost::Late)]);
    rig.clock.advance(ms(TIMES.restart_ms));
    rig.begin(start).unwrap();
    let now = rig.clock.now();
    rig.flow.heard(old, Ok(level(1)), now);
    rig.flow.heard(old, Err(IpcError::Closed), now);
    assert_eq!(rig.heard(), [], "the live worker stays");
    rig.flow.ask(stop(2), Wait::Each(WAIT)).unwrap();
    assert_eq!(rig.script.calls().last(), Some(&Call::Voice(stop(2))));
}
