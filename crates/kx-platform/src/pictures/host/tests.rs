//! The media job over a scripted worker: it starts as the config says, a reply that does not fit the draw ends the
//! worker, the worker's reasons become the port's errors, asks the worker would refuse start nothing, and a side past
//! the frame's limit is clamped. The core's own rules are tested in `worker::host`.

mod dib;

use super::*;
use crate::pictures::times;
use crate::worker::Launch;
use crate::worker::fake::{Act, Call, Hands, Script};
use kx_ipc::RGBA_BYTES;
use kx_module_api::ports::PictureConfig;
use std::sync::Arc;
use std::time::Duration;

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

fn ms(n: u32) -> Duration {
    Duration::from_millis(u64::from(n))
}

fn host(script: &Script<MediaFrames>, clock: &Hands) -> Host<Script<MediaFrames>, MediaFrames> {
    let (spawner, clock) = (Arc::new(script.clone()), Arc::new(clock.clone()));
    Host::new(spawner, clock, times(&CFG), Launch::media(&CFG))
}

fn ask(side: u32) -> PictureAsk {
    PictureAsk {
        codec: ImageCodec::Png,
        bytes: Redacted::new(vec![1, 2, 3]),
        side,
    }
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
fn a_late_draw_is_a_timeout_and_the_worker_rests() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    script.act(Act::Silent);
    assert_eq!(host.draw(ask(SIDE)), Err(PictureError::Timeout));
    assert_eq!(host.draw(ask(SIDE)), Err(down(ms(CFG.restart_ms))));
    clock.advance(ms(CFG.restart_ms));
    assert!(host.draw(ask(SIDE)).is_ok());
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
