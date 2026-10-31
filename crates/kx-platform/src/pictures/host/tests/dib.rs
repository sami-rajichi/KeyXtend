//! The host's DIB asks over a scripted worker: the PNG and its cap go out, only the reply's length is checked, a lie
//! or a late reply ends the worker, and an ask the worker would refuse never starts one.

use super::*;
use kx_module_api::ports::DibAsk;

/// A DIB ask of a 3-byte PNG, at most `max_bytes` long.
fn dib(max_bytes: u64) -> DibAsk {
    DibAsk {
        png: Redacted::new(vec![1, 2, 3]),
        max_bytes,
    }
}

/// The largest image a frame carries, as an ask's cap.
fn largest() -> u32 {
    u32::try_from(MAX_IMAGE_BYTES).unwrap()
}

#[test]
fn a_dib_goes_out_with_its_cap_and_its_bytes_come_back() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    script.act(Act::Dib(44));
    let made = host.dib(dib(100)).map(Redacted::into_inner);
    assert_eq!(made, Ok(vec![0; 44]));
    let asked = Call::Dib {
        id: 1,
        max_bytes: 100,
    };
    assert_eq!(script.calls(), [asked, reply()]);
    script.act(Act::Dib(100));
    assert!(host.dib(dib(100)).is_ok(), "a DIB at its cap");
    assert_eq!(script.spawns(), 1, "draws and DIBs share the worker");
}

#[test]
fn a_cap_past_the_largest_image_is_clamped_to_it() {
    let (script, clock) = (Script::default(), Hands::default());
    script.act(Act::Dib(4));
    host(&script, &clock).dib(dib(u64::MAX)).unwrap();
    let max_bytes = largest();
    assert_eq!(script.calls()[0], Call::Dib { id: 1, max_bytes });
}

#[test]
fn a_refusal_is_passed_on_and_the_worker_kept() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    for why in [Why::Refused, Why::TooBig] {
        script.act(Act::Fail(why));
        assert_eq!(host.dib(dib(100)), Err(refusal(why)));
    }
    assert!(!script.calls().contains(&Call::End));
}

#[test]
fn a_dib_past_its_cap_late_stray_or_of_another_kind_ends_the_worker() {
    let cases = [
        (Act::Dib(101), PictureError::Gone),
        (Act::Silent, PictureError::Timeout),
        (Act::Draw(1, 1), PictureError::Gone),
        (
            Act::Raw(Reply::Dib {
                id: 9,
                bytes: vec![0; 4],
            }),
            PictureError::Gone,
        ),
    ];
    for (act, want) in cases {
        let (script, clock) = (Script::default(), Hands::default());
        let mut host = host(&script, &clock);
        script.act(act.clone());
        assert_eq!(host.dib(dib(100)), Err(want), "{act:?}");
        assert!(script.calls().ends_with(&[reply(), Call::End]), "{act:?}");
        assert_eq!(host.dib(dib(100)), Err(down(ms(CFG.restart_ms))));
    }
}

#[test]
fn a_dib_reply_to_a_draw_ends_the_worker() {
    let (script, clock) = (Script::default(), Hands::default());
    script.act(Act::Dib(4));
    let mut host = host(&script, &clock);
    assert_eq!(host.draw(ask(SIDE)), Err(PictureError::Gone));
    assert!(script.calls().ends_with(&[reply(), Call::End]));
}

#[test]
fn an_ask_the_worker_would_refuse_starts_no_worker() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    assert_eq!(host.dib(dib(0)), Err(PictureError::Refused), "no room");
    let empty = DibAsk {
        png: Redacted::new(Vec::new()),
        ..dib(100)
    };
    assert_eq!(host.dib(empty), Err(PictureError::Refused));
    let huge = DibAsk {
        png: Redacted::new(vec![0; MAX_IMAGE_BYTES + 1]),
        ..dib(100)
    };
    assert_eq!(host.dib(huge), Err(PictureError::TooBig));
    assert_eq!(script.tries(), 0);
}
