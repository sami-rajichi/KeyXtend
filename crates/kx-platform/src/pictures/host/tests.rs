//! The host over a scripted worker: it starts on demand, draws one at a time, ends a late or lying worker and is down a
//! growing backoff before the next start, refuses asks the worker would refuse, and stops when idle; nothing sleeps.

mod dib;

use super::*;
use crate::pictures::Launch;
use crate::pictures::fake::{Act, Call, Hands, Script};
use kx_ipc::{IpcError, MAX_IMAGE_BYTES, MAX_SIDE, RGBA_BYTES, Reply, Why};
use kx_module_api::Redacted;
use kx_module_api::ports::ImageCodec;

/// Test values; the real ones come from the clipboard module's settings.
const CFG: PictureConfig = PictureConfig {
    deadline_ms: 2_000,
    idle_s: 30,
    memory_mb: 256,
    max_pixels: 40_000_000,
    restart_ms: 1_000,
    restart_max_ms: 4_000,
};

/// The longest side the tests ask for.
const SIDE: u32 = 4;
/// The smallest step of the clock.
const TICK: Duration = Duration::from_micros(1);
/// The exit code of a worker that died.
const DIED: u32 = 3;

fn ms(n: u32) -> Duration {
    Duration::from_millis(u64::from(n))
}

fn host(script: &Script, clock: &Hands) -> Host<Script> {
    Host::new(Arc::new(script.clone()), Arc::new(clock.clone()), CFG)
}

fn ask(side: u32) -> PictureAsk {
    PictureAsk {
        codec: ImageCodec::Png,
        bytes: Redacted::new(vec![1, 2, 3]),
        side,
    }
}

fn draw(id: u32) -> Call {
    Call::Draw { id, side: SIDE }
}

/// A draw refused as the worker rests, `retry_in` before it may start again.
fn down(retry_in: Duration) -> PictureError {
    PictureError::Down { retry_in }
}

/// A wait for a reply, as long as the deadline.
fn reply() -> Call {
    Call::Reply {
        within: ms(CFG.deadline_ms),
    }
}

/// A wait for the exit code, as long as the deadline.
fn code() -> Call {
    Call::Code {
        within: ms(CFG.deadline_ms),
    }
}

/// A look at the exit code with no wait, before each later draw.
fn peek() -> Call {
    Call::Code {
        within: Duration::ZERO,
    }
}

#[test]
fn nothing_starts_before_the_first_draw_and_one_worker_draws_each_in_turn() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    assert_eq!(script.tries(), 0);
    script.act(Act::Draw(2, 1));
    script.act(Act::Draw(1, 2));
    let first = host.draw(ask(SIDE)).unwrap();
    assert_eq!((first.width(), first.height()), (2, 1));
    let second = host.draw(ask(SIDE)).unwrap();
    assert_eq!((second.width(), second.height()), (1, 2));
    assert_eq!(script.spawns(), 1);
    assert_eq!(script.calls(), [draw(1), reply(), peek(), draw(2), reply()]);
}

#[test]
fn the_worker_starts_as_the_config_says() {
    let (script, clock) = (Script::default(), Hands::default());
    host(&script, &clock).draw(ask(SIDE)).unwrap();
    let args = ["media", "--max-pixels", "40000000"]
        .map(String::from)
        .to_vec();
    let memory_mb = CFG.memory_mb;
    assert_eq!(script.launches(), [Launch { args, memory_mb }]);
    let none = Launch::media(&PictureConfig {
        max_pixels: 0,
        ..CFG
    });
    assert_eq!(none.args[2], "1", "the worker refuses 0");
}

#[test]
fn a_late_worker_is_ended_and_is_down_until_its_backoff_ends() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    script.act(Act::Silent);
    assert_eq!(host.draw(ask(SIDE)), Err(PictureError::Timeout));
    assert!(script.calls().ends_with(&[reply(), Call::End]));
    clock.advance(ms(CFG.restart_ms).saturating_sub(TICK));
    assert_eq!(host.draw(ask(SIDE)), Err(down(TICK)), "the wait left");
    assert_eq!(script.tries(), 1, "no start within the backoff");
    clock.advance(TICK);
    assert!(host.draw(ask(SIDE)).is_ok());
    assert_eq!(script.spawns(), 2);
}

#[test]
fn the_backoff_doubles_up_to_its_cap_and_a_good_draw_resets_it() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    for wait in [1_000, 2_000, 4_000, 4_000] {
        script.act(Act::Silent);
        assert_eq!(host.draw(ask(SIDE)), Err(PictureError::Timeout));
        clock.advance(ms(wait).saturating_sub(TICK));
        assert_eq!(host.draw(ask(SIDE)), Err(down(TICK)), "{wait}");
        clock.advance(TICK);
    }
    assert!(host.draw(ask(SIDE)).is_ok());
    script.act(Act::Silent);
    assert_eq!(host.draw(ask(SIDE)), Err(PictureError::Timeout));
    clock.advance(ms(CFG.restart_ms));
    assert!(host.draw(ask(SIDE)).is_ok(), "back to the first wait");
}

#[test]
fn bad_replies_are_refused_and_end_the_worker() {
    let long = vec![0; RGBA_BYTES + 1];
    let cases = [
        Act::Raw(Reply::Failed {
            id: 9,
            why: Why::Refused,
        }),
        Act::Draw(SIDE + 1, 1),
        Act::Raw(Reply::Pixels {
            id: 1,
            width: 1,
            height: 1,
            rgba: long,
        }),
        Act::Broken(IpcError::BadMagic),
        Act::Broken(IpcError::Short),
        Act::Broken(IpcError::Closed),
    ];
    for act in cases {
        let (script, clock) = (Script::default(), Hands::default());
        let mut host = host(&script, &clock);
        script.act(act.clone());
        assert_eq!(host.draw(ask(SIDE)), Err(PictureError::Gone), "{act:?}");
        assert!(script.calls().ends_with(&[Call::End]), "{act:?}");
        let closed = matches!(act, Act::Broken(IpcError::Closed));
        assert_eq!(
            script.calls().contains(&code()),
            closed,
            "{act:?} reads the code"
        );
        let rest = Err(down(ms(CFG.restart_ms)));
        assert_eq!(host.draw(ask(SIDE)), rest, "{act:?} is down");
        assert_eq!(script.tries(), 1, "{act:?} backs off");
    }
}

#[test]
fn a_failed_draw_is_told_and_keeps_the_worker() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    let cases = [
        (Why::Refused, PictureError::Refused),
        (Why::TooBig, PictureError::TooBig),
        (Why::Internal, PictureError::Refused),
    ];
    for (why, error) in cases {
        script.act(Act::Fail(why));
        assert_eq!(host.draw(ask(SIDE)), Err(error), "{why:?}");
    }
    assert!(host.draw(ask(SIDE)).is_ok());
    assert_eq!(script.spawns(), 1);
    assert!(!script.calls().contains(&Call::End));
}

#[test]
fn asks_the_worker_would_refuse_never_start_it() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    let empty = PictureAsk {
        bytes: Redacted::new(Vec::new()),
        ..ask(SIDE)
    };
    assert_eq!(host.draw(empty), Err(PictureError::Refused));
    assert_eq!(host.draw(ask(0)), Err(PictureError::Refused));
    let huge = PictureAsk {
        bytes: Redacted::new(vec![0; MAX_IMAGE_BYTES + 1]),
        ..ask(SIDE)
    };
    assert_eq!(host.draw(huge), Err(PictureError::TooBig));
    assert_eq!(script.tries(), 0);
}

#[test]
fn a_side_past_the_frame_limit_is_clamped() {
    let (script, clock) = (Script::default(), Hands::default());
    host(&script, &clock).draw(ask(MAX_SIDE + 1)).unwrap();
    let first = script.calls()[0].clone();
    assert_eq!(
        first,
        Call::Draw {
            id: 1,
            side: MAX_SIDE
        }
    );
}

#[test]
fn an_idle_worker_says_bye_and_stops_with_no_backoff() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    let idle = Duration::from_secs(u64::from(CFG.idle_s));
    assert_eq!(host.wake_in(), None, "no worker, no wake");
    host.draw(ask(SIDE)).unwrap();
    assert_eq!(host.wake_in(), Some(idle));
    clock.advance(idle.saturating_sub(TICK));
    host.idle();
    assert_eq!(host.wake_in(), Some(TICK));
    assert!(!script.calls().contains(&Call::Bye));
    clock.advance(TICK);
    host.idle();
    assert!(script.calls().ends_with(&[Call::Bye, code(), Call::End]));
    assert_eq!(host.wake_in(), None);
    host.draw(ask(SIDE)).unwrap();
    assert_eq!(script.spawns(), 2, "a fresh start at once");
}

#[test]
fn a_draw_past_the_idle_time_starts_a_fresh_worker() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    host.draw(ask(SIDE)).unwrap();
    clock.advance(Duration::from_secs(u64::from(CFG.idle_s)));
    host.draw(ask(SIDE)).unwrap();
    assert_eq!(script.spawns(), 2);
    assert!(script.calls().contains(&Call::Bye));
}

#[test]
fn a_worker_that_quit_between_draws_is_replaced_with_no_backoff() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    host.draw(ask(SIDE)).unwrap();
    script.quit(DIED);
    assert!(
        host.draw(ask(SIDE)).is_ok(),
        "no blame for a death between draws"
    );
    assert_eq!(script.spawns(), 2);
    let calls = [draw(1), reply(), peek(), Call::End, draw(2), reply()];
    assert_eq!(script.calls(), calls);
}

#[test]
fn a_worker_that_does_not_start_is_down_and_backs_off() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    script.refuse(2);
    let first = down(ms(CFG.restart_ms));
    assert_eq!(host.draw(ask(SIDE)), Err(first), "no image was tried");
    assert_eq!(host.draw(ask(SIDE)), Err(first));
    assert_eq!(script.tries(), 1);
    clock.advance(ms(CFG.restart_ms));
    assert_eq!(host.draw(ask(SIDE)), Err(down(ms(CFG.restart_ms * 2))));
    assert_eq!(script.tries(), 2);
    clock.advance(ms(CFG.restart_ms * 2));
    assert!(host.draw(ask(SIDE)).is_ok());
    assert_eq!(script.tries(), 3);
}

#[test]
fn dropping_the_host_stops_its_worker() {
    let (script, clock) = (Script::default(), Hands::default());
    drop(host(&script, &clock));
    assert!(script.calls().is_empty(), "no worker, nothing to stop");
    let mut host = host(&script, &clock);
    host.draw(ask(SIDE)).unwrap();
    drop(host);
    assert!(script.calls().ends_with(&[Call::Bye, code(), Call::End]));
}
