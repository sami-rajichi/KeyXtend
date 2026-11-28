//! The host's encodes over a scripted worker: the pixels go out with their sides, a PNG within `png_most` of them comes
//! back unread, the worker's reasons become the port's, and a PNG past its bound, a late or a stray reply ends the
//! worker.

use super::*;
use kx_ipc::{BGRX_BYTES, png_most};
use kx_module_api::ports::{EncodeAsk, Shot};

/// The test picture's sides.
const WIDTH: u32 = 3;
const HEIGHT: u32 = 2;

/// The raw bytes of the test picture.
fn raw_len() -> usize {
    (WIDTH * HEIGHT) as usize * BGRX_BYTES
}

/// An encode of the test picture.
fn encode() -> EncodeAsk {
    let shot = Shot::new(WIDTH, HEIGHT, vec![9; raw_len()]).unwrap();
    EncodeAsk { shot }
}

#[test]
fn an_encode_goes_out_with_its_sides_and_its_png_comes_back() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    script.act(Act::Png(png_most(raw_len())));
    let made = host.encode(&encode()).map(|png| png.into_inner().len());
    assert_eq!(made, Ok(png_most(raw_len())), "a PNG at its bound");
    let asked = Call::Encode {
        id: 1,
        width: WIDTH,
        height: HEIGHT,
    };
    assert_eq!(script.calls(), [asked, reply()]);
    assert!(
        host.draw(ask(SIDE)).is_ok(),
        "encodes and draws share the worker"
    );
    assert_eq!(script.spawns(), 1);
}

#[test]
fn a_refusal_is_passed_on_and_the_worker_kept() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    for why in [Why::Refused, Why::TooBig, Why::Internal] {
        script.act(Act::Fail(why));
        assert_eq!(host.encode(&encode()), Err(refusal(why)), "{why:?}");
    }
    assert!(!script.calls().contains(&Call::End));
}

#[test]
fn a_png_past_its_bound_late_stray_or_of_another_kind_ends_the_worker() {
    let cases = [
        (Act::Png(png_most(raw_len()) + 1), PictureError::Gone),
        (Act::Silent, PictureError::Timeout),
        (Act::Draw(1, 1), PictureError::Gone),
        (Act::Dib(4), PictureError::Gone),
        (
            Act::Raw(Reply::Png {
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
        assert_eq!(host.encode(&encode()), Err(want), "{act:?}");
        assert!(script.calls().ends_with(&[reply(), Call::End]), "{act:?}");
        assert_eq!(host.encode(&encode()), Err(down(ms(CFG.restart_ms))));
    }
}

#[test]
fn a_png_reply_to_a_draw_ends_the_worker() {
    let (script, clock) = (Script::default(), Hands::default());
    script.act(Act::Png(4));
    let mut host = host(&script, &clock);
    assert_eq!(host.draw(ask(SIDE)), Err(PictureError::Gone));
    assert!(script.calls().ends_with(&[reply(), Call::End]));
}
