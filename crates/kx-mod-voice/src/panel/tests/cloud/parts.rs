//! The card and a key page keep the panel's height (ADR-0008): a card takes two pages, the voice then the price, which
//! the pager turns, and a key page takes one, with a typed address's lines in the order address, model, key (owner,
//! 2026-11-21).

use kx_module_api::Redacted;
use kx_module_api::ports::{
    CardPart, CloudButton, CloudLine, Pages, ServiceName, VoiceClick, VoicePanel, VoiceSheet,
};

use super::super::super::rig::Rig;
use super::{act, on_cloud, put, shown};
use crate::services::{
    self,
    tests::shipped::{entry, first},
};

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

/// The shown key page's lines in page order, and the one taking the letters.
fn lines(sheet: VoiceSheet) -> (Vec<CloudLine>, CloudLine) {
    match sheet {
        VoiceSheet::Service(s) => (s.lines, s.typing),
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
    act(&mut rig, &shown("groq"), CloudButton::Facts);
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
    let custom = services::table().unwrap().get("custom").unwrap();
    let on = format!("{}\n{}", entry(first(), ""), entry(custom, ""));
    put(&mut rig, "services", &on);
    act(&mut rig, name, CloudButton::Lines);
    rig
}

#[test]
fn a_key_page_takes_one_page_with_a_typed_addresss_lines_first() {
    use CloudLine::{Address, Key, Model};
    let one = Pages { at: 0, count: 1 };
    let groq = turned(&mut on_lines(&shown("groq")), 0, lines);
    assert_eq!(groq, ((vec![Key, Model], Key), one), "Groq's");
    let mut rig = on_lines(&ServiceName::Custom);
    let typed = vec![Address, Model, Key];
    let first = ((typed.clone(), Address), one);
    assert_eq!(turned(&mut rig, 0, lines), first, "the address aimed");
    assert_eq!(turned(&mut rig, 1, lines), first, "no second page");
    rig.click(&VoiceClick::Caret {
        line: Key,
        at: Redacted::new(0),
    });
    let aimed = turned(&mut rig, 0, lines);
    assert_eq!(aimed, ((typed, Key), one), "a click aims");
}
