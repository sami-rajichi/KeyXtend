//! Edge case 34: a text longer than `text_cap` is typed in pieces under the cap, never split inside a character, and
//! its Undo erases past the cap in pieces too, where a picked word would be refused.

use super::{BREAK, Mic, done, erase_gap};
use crate::told::BKSP;
use crate::typing::{tap, unicode};
use kx_mod_keyboard::TEXT_CAP;
use kx_platform_fake::Stroke;

/// The smallest `text_cap` the settings take.
fn cap() -> u16 {
    *TEXT_CAP.start()
}

/// The module at the smallest `text_cap`.
fn capped() -> Mic {
    Mic::quiet(Some(("text_cap", i64::from(cap()).into())))
}

/// The text of each stroke, which must all be Unicode text.
fn texts(strokes: &[Stroke]) -> Vec<String> {
    let text = |s: &Stroke| match s {
        Stroke::Text(t, _) => t.expose().clone(),
        other => panic!("not text: {other:?}"),
    };
    strokes.iter().map(text).collect()
}

#[test]
fn a_text_longer_than_text_cap_is_typed_in_pieces_under_the_cap() {
    let m = capped();
    let cap = usize::from(cap());
    let t = "ab".repeat(cap + cap / 4);
    m.ask(1, 0, &t, 0);
    let want = [
        unicode(&t[..cap]),
        unicode(&t[cap..2 * cap]),
        unicode(&t[2 * cap..]),
    ];
    assert_eq!(m.b.rig.fakes.injector.strokes(), want);
    let count = u32::try_from(t.len()).unwrap();
    assert_eq!(m.heard(), [BREAK, done(1, count, true)]);
}

#[test]
fn arabic_pieces_never_split_a_character() {
    let m = capped();
    let t = "سَلامٌ عَلَيْكُم ".repeat(usize::from(cap()) / 8);
    assert!(t.chars().count() > usize::from(cap()));
    m.ask(1, 0, &t, 0);
    let pieces = texts(&m.b.rig.fakes.injector.strokes());
    assert!(pieces.len() > 1, "in pieces");
    assert!(
        pieces
            .iter()
            .all(|p| p.chars().count() <= usize::from(cap()))
    );
    assert_eq!(pieces.concat(), t, "nothing lost or split");
    let count = u32::try_from(t.chars().count()).unwrap();
    assert_eq!(m.heard(), [BREAK, done(1, count, true)]);
}

#[test]
fn an_undo_past_text_cap_erases_every_character() {
    let m = capped();
    let erase = u32::from(cap()) * 2 + 3;
    m.ask(2, erase, "", 0);
    let strokes = m.b.rig.fakes.injector.strokes();
    assert_eq!(strokes.len(), usize::try_from(erase).unwrap());
    assert!(strokes.iter().all(|s| *s == tap(BKSP)));
    assert_eq!(m.heard(), [BREAK, done(2, 0, true)]);
}

#[test]
fn an_undo_waits_erase_gap_ms_after_each_backspace_but_the_last_whatever_the_asks_gap() {
    let m = capped();
    m.ask(3, 3, "", 0);
    assert_eq!(m.b.rig.fakes.injector.strokes(), vec![tap(BKSP); 3]);
    let waits = m.b.rig.fakes.pause.waits();
    let gaps: Vec<_> = waits.into_iter().map(|w| w.1).collect();
    assert_eq!(gaps, [erase_gap(); 2]);
}
