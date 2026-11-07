//! The core over a scripted media worker: it starts on demand and answers one ask at a time, ends a late, stray or
//! broken worker and is down a doubling backoff before the next start, never takes a reply to an earlier ask, starts
//! nothing for an ask refused first, and stops when idle; nothing sleeps.

use super::*;
use crate::worker::MediaFrames;
use crate::worker::fake::{Act, Call, Hands, Script};
use kx_ipc::{Ask, Codec, IpcError, Reply, Why};
use std::borrow::Cow;

mod props;

/// Test values; the real ones come from each port's config.
const TIMES: Times = Times {
    deadline_ms: 2_000,
    idle_s: 30,
    restart_ms: 1_000,
    restart_max_ms: 4_000,
};

/// The smallest step of the clock.
const TICK: Duration = Duration::from_micros(1);
/// The longest side a test draw asks for.
const SIDE: u32 = 1;
/// The exit code of a worker that died.
const DIED: u32 = 3;

/// The host the tests drive.
type Media = Host<Script<MediaFrames>, MediaFrames>;

/// Why a test ask has no answer: the host's reason, or the worker's refusal.
#[derive(Debug, PartialEq, Eq)]
enum Miss {
    Lost(Lost),
    Refused,
}

impl From<Lost> for Miss {
    fn from(lost: Lost) -> Self {
        Self::Lost(lost)
    }
}

/// The command line the tests start with.
fn launch() -> Launch {
    Launch {
        args: vec!["job".to_owned()],
        memory_mb: 64,
    }
}

fn host(script: &Script<MediaFrames>, clock: &Hands) -> Media {
    let (spawner, clock) = (Arc::new(script.clone()), Arc::new(clock.clone()));
    Host::new(spawner, clock, TIMES, launch())
}

/// One draw: the id its pixels answer, a refusal for a failure, and no fit for a DIB.
fn ping(host: &mut Media) -> Result<u32, Miss> {
    let make = |id| {
        let bytes = Cow::Borrowed(&[1_u8][..]);
        Ok(Ask::Draw {
            id,
            codec: Codec::Png,
            side: SIDE,
            bytes,
        })
    };
    host.call(make, |reply| match reply {
        Reply::Pixels { id, .. } => Some(Ok(id)),
        Reply::Failed { .. } => Some(Err(Miss::Refused)),
        Reply::Dib { .. } => None,
    })
}

fn ms(n: u32) -> Duration {
    Duration::from_millis(u64::from(n))
}

/// The ask with `id`, as the worker got it.
fn asked(id: u32) -> Call {
    Call::Draw { id, side: SIDE }
}

/// An ask refused as the worker rests, `retry_in` before it may start again.
fn down(retry_in: Duration) -> Result<u32, Miss> {
    Err(Miss::Lost(Lost::Down { retry_in }))
}

/// A wait for a reply, as long as the deadline.
fn reply() -> Call {
    let within = ms(TIMES.deadline_ms);
    Call::Reply { within }
}

/// A wait for the exit code, as long as the deadline.
fn code() -> Call {
    let within = ms(TIMES.deadline_ms);
    Call::Code { within }
}

/// A look at the exit code with no wait, before each later ask.
fn peek() -> Call {
    let within = Duration::ZERO;
    Call::Code { within }
}

#[test]
fn nothing_starts_before_the_first_ask_and_one_worker_answers_each_in_turn() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    assert_eq!(script.tries(), 0);
    assert_eq!(ping(&mut host), Ok(1));
    assert_eq!(ping(&mut host), Ok(2));
    assert_eq!(script.spawns(), 1);
    assert_eq!(script.launches(), [launch()], "as the host was told");
    let calls = [asked(1), reply(), peek(), asked(2), reply()];
    assert_eq!(script.calls(), calls);
}

#[test]
fn a_late_worker_is_ended_and_is_down_until_its_backoff_ends() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    script.act(Act::Silent);
    assert_eq!(ping(&mut host), Err(Miss::Lost(Lost::Late)));
    assert!(script.calls().ends_with(&[reply(), Call::End]));
    clock.advance(ms(TIMES.restart_ms).saturating_sub(TICK));
    assert_eq!(ping(&mut host), down(TICK), "the wait left");
    assert_eq!(script.tries(), 1, "no start within the backoff");
    clock.advance(TICK);
    assert!(ping(&mut host).is_ok());
    assert_eq!(script.spawns(), 2);
}

#[test]
fn the_backoff_doubles_up_to_its_cap_and_a_good_answer_resets_it() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    let late = Err(Miss::Lost(Lost::Late));
    for wait in [1_000, 2_000, 4_000, 4_000] {
        script.act(Act::Silent);
        assert_eq!(ping(&mut host), late);
        clock.advance(ms(wait).saturating_sub(TICK));
        assert_eq!(ping(&mut host), down(TICK), "{wait}");
        clock.advance(TICK);
    }
    assert!(ping(&mut host).is_ok());
    script.act(Act::Silent);
    assert_eq!(ping(&mut host), late);
    clock.advance(ms(TIMES.restart_ms));
    assert!(ping(&mut host).is_ok(), "back to the first wait");
}

#[test]
fn stray_broken_or_unfit_replies_end_the_worker() {
    let stray = Reply::Failed {
        id: 9,
        why: Why::Refused,
    };
    let cases = [
        Act::Raw(stray),
        Act::Dib(1),
        Act::Broken(IpcError::BadMagic),
        Act::Broken(IpcError::Short),
        Act::Broken(IpcError::Closed),
    ];
    for act in cases {
        let (script, clock) = (Script::default(), Hands::default());
        let mut host = host(&script, &clock);
        script.act(act.clone());
        assert_eq!(ping(&mut host), Err(Miss::Lost(Lost::Gone)), "{act:?}");
        assert!(script.calls().ends_with(&[Call::End]), "{act:?}");
        let closed = matches!(act, Act::Broken(IpcError::Closed));
        let reads = script.calls().contains(&code());
        assert_eq!(reads, closed, "{act:?} reads the code");
        assert_eq!(ping(&mut host), down(ms(TIMES.restart_ms)), "{act:?}");
        assert_eq!(script.tries(), 1, "{act:?} backs off");
    }
}

#[test]
fn a_reply_to_an_ask_that_timed_out_is_never_taken() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    script.act(Act::Silent);
    assert_eq!(ping(&mut host), Err(Miss::Lost(Lost::Late)));
    clock.advance(ms(TIMES.restart_ms));
    let stale = Reply::Pixels {
        id: 1,
        width: 1,
        height: 1,
        rgba: vec![0; 4],
    };
    script.act(Act::Raw(stale));
    assert_eq!(ping(&mut host), Err(Miss::Lost(Lost::Gone)), "ask 2");
    assert!(script.calls().ends_with(&[asked(2), reply(), Call::End]));
}

#[test]
fn a_refusal_is_an_answer_and_keeps_the_worker() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    script.act(Act::Fail(Why::Internal));
    assert_eq!(ping(&mut host), Err(Miss::Refused));
    assert_eq!(ping(&mut host), Ok(2));
    assert_eq!(script.spawns(), 1);
    assert!(!script.calls().contains(&Call::End));
}

#[test]
fn an_ask_refused_before_the_start_starts_nothing_and_takes_no_id() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    let refused = host.call(
        |_| Err::<Ask<'static>, _>(Miss::Refused),
        |_| None::<Result<u32, _>>,
    );
    assert_eq!(refused, Err(Miss::Refused));
    assert_eq!(script.tries(), 0);
    assert_eq!(ping(&mut host), Ok(1));
}

#[test]
fn an_idle_worker_says_bye_and_stops_with_no_backoff() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    let idle = Duration::from_secs(u64::from(TIMES.idle_s));
    assert_eq!(host.wake_in(), None, "no worker, no wake");
    ping(&mut host).unwrap();
    assert_eq!(host.wake_in(), Some(idle));
    clock.advance(idle.saturating_sub(TICK));
    host.idle();
    assert_eq!(host.wake_in(), Some(TICK));
    assert!(!script.calls().contains(&Call::Bye));
    clock.advance(TICK);
    host.idle();
    assert!(script.calls().ends_with(&[Call::Bye, code(), Call::End]));
    assert_eq!(host.wake_in(), None);
    ping(&mut host).unwrap();
    assert_eq!(script.spawns(), 2, "a fresh start at once");
}

#[test]
fn an_ask_past_the_idle_time_starts_a_fresh_worker() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    ping(&mut host).unwrap();
    clock.advance(Duration::from_secs(u64::from(TIMES.idle_s)));
    ping(&mut host).unwrap();
    assert_eq!(script.spawns(), 2);
    assert!(script.calls().contains(&Call::Bye));
}

#[test]
fn a_worker_that_quit_between_asks_is_replaced_with_no_backoff() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    ping(&mut host).unwrap();
    script.quit(DIED);
    assert_eq!(ping(&mut host), Ok(2), "no blame for a death between asks");
    assert_eq!(script.spawns(), 2);
    let calls = [asked(1), reply(), peek(), Call::End, asked(2), reply()];
    assert_eq!(script.calls(), calls);
}

#[test]
fn a_worker_that_does_not_start_is_down_and_backs_off() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    script.refuse(2);
    let first = down(ms(TIMES.restart_ms));
    assert_eq!(ping(&mut host), first, "nothing was tried");
    assert_eq!(ping(&mut host), first);
    assert_eq!(script.tries(), 1);
    clock.advance(ms(TIMES.restart_ms));
    assert_eq!(ping(&mut host), down(ms(TIMES.restart_ms * 2)));
    assert_eq!(script.tries(), 2);
    clock.advance(ms(TIMES.restart_ms * 2));
    assert!(ping(&mut host).is_ok());
    assert_eq!(script.tries(), 3);
}

#[test]
fn dropping_the_host_stops_its_worker() {
    let (script, clock) = (Script::default(), Hands::default());
    drop(host(&script, &clock));
    assert!(script.calls().is_empty(), "no worker, nothing to stop");
    let mut host = host(&script, &clock);
    ping(&mut host).unwrap();
    drop(host);
    assert!(script.calls().ends_with(&[Call::Bye, code(), Call::End]));
}
