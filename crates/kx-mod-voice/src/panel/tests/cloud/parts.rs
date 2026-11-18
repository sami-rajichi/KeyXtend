//! The card and a key page keep the panel's height (ADR-0008): a card takes two pages, the voice then the price, and a
//! key page with a typed address takes two, the key then the address and model; the pager turns them.

use kx_module_api::ports::{
    CardPart, CloudButton, LinesPart, Pages, ServiceName, VoiceClick, VoicePanel, VoiceSheet,
};

use super::super::super::rig::Rig;
use super::{act, named, on_cloud, put};

/// The shown sheet's part and the pages, after `turns` page clicks (back when negative).
fn turned<T>(rig: &mut Rig, turns: i8, part: impl Fn(VoiceSheet) -> T) -> (T, Pages) {
    for _ in 0..turns.unsigned_abs() {
        rig.click(&VoiceClick::Page { back: turns < 0 });
    }
    let VoicePanel { sheet, pages, .. } = rig.view().unwrap();
    (part(sheet), pages)
}

/// The shown card's part.
fn card(sheet: VoiceSheet) -> CardPart {
    match sheet {
        VoiceSheet::Card(c) => c.part,
        other => panic!("not a card: {other:?}"),
    }
}

/// The shown key page's part.
fn lines(sheet: VoiceSheet) -> Option<LinesPart> {
    match sheet {
        VoiceSheet::Service(s) => s.part,
        other => panic!("not a key page: {other:?}"),
    }
}

/// Page `at` of two.
const fn of_two(at: u16) -> Pages {
    Pages { at, count: 2 }
}

#[test]
fn a_card_takes_two_pages_the_voice_then_the_price() {
    let mut rig = on_cloud("");
    act(&mut rig, &named("Groq"), CloudButton::Facts);
    assert_eq!(turned(&mut rig, 0, card), (CardPart::Voice, of_two(0)));
    assert_eq!(turned(&mut rig, 1, card), (CardPart::Price, of_two(1)));
    assert_eq!(
        turned(&mut rig, 1, card),
        (CardPart::Price, of_two(1)),
        "no page past the last"
    );
    assert_eq!(turned(&mut rig, -1, card), (CardPart::Voice, of_two(0)));
}

/// Groq and Custom on, on the key page of row `name`.
fn on_lines(name: &ServiceName) -> Rig {
    let mut rig = on_cloud("");
    let on = "groq = { on = 2026-11-01, terms = 1 }\ncustom = { on = 2026-11-01, terms = 1 }";
    put(&mut rig, "services", on);
    act(&mut rig, name, CloudButton::Lines);
    rig
}

#[test]
fn a_key_page_takes_two_pages_only_with_a_typed_address() {
    let one = Pages { at: 0, count: 1 };
    assert_eq!(
        turned(&mut on_lines(&named("Groq")), 0, lines),
        (None, one),
        "Groq's"
    );
    let mut rig = on_lines(&ServiceName::Custom);
    assert_eq!(
        turned(&mut rig, 0, lines),
        (Some(LinesPart::Key), of_two(0))
    );
    assert_eq!(
        turned(&mut rig, 1, lines),
        (Some(LinesPart::Model), of_two(1))
    );
}
