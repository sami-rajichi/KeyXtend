//! Text: it opens where clicked, takes letters at its caret, ends on Enter or a click elsewhere, drops an empty text,
//! keeps Arabic as typed, reading right to left, and never prints.

use kx_module_api::caret::Side;
use kx_module_api::ports::PanelKey;

use super::*;

/// The text being typed: its place, its text, its caret in UTF-16 units, and whether it reads right to left.
fn typed(c: &Canvas) -> Option<(Point, String, u32, bool)> {
    let t = c.view().typing?;
    Some((t.at, t.text.expose().clone(), *t.caret.expose(), t.rtl))
}

/// An editor typing `text` at (30, 40).
fn typing(text: &str) -> Canvas {
    let mut c = with(Tool::Text);
    c.press(pt(30, 40), SAME);
    c.write(text);
    c
}

/// Letters for the line.
fn letters(text: &str) -> PanelKey {
    PanelKey::Text(Redacted::new(text.to_owned()))
}

#[test]
fn text_opens_where_clicked_and_takes_letters_at_its_caret() {
    let mut c = with(Tool::Text);
    assert!(!c.typing());
    assert_eq!(c.press(pt(30, 40), SAME), Did::Show);
    assert!(c.typing());
    assert_eq!(typed(&c), Some((pt(30, 40), String::new(), 0, false)));
    assert_eq!(c.write("Helo"), Did::Show);
    assert_eq!(c.arrow(Side::Left), Did::Show);
    assert_eq!(c.write("l"), Did::Show);
    assert_eq!(typed(&c), Some((pt(30, 40), "Hello".into(), 4, false)));
    assert!(
        drawn(&c).is_empty(),
        "nothing joins the drawings while it is typed"
    );
}

#[test]
fn backspace_and_delete_act_at_the_caret() {
    let mut c = typing("abcd");
    assert_eq!(c.caret(2), Did::Show);
    assert_eq!(c.erase(), Did::Show);
    assert_eq!(typed(&c).map(|t| (t.1, t.2)), Some(("acd".into(), 1)));
    assert_eq!(c.delete(), Did::Show);
    assert_eq!(typed(&c).map(|t| (t.1, t.2)), Some(("ad".into(), 1)));
    c.caret(0);
    assert_eq!(c.erase(), Did::Stay, "nothing before the caret");
    c.caret(2);
    assert_eq!(c.delete(), Did::Stay, "nothing after it");
}

#[test]
fn enter_or_a_click_elsewhere_ends_the_text() {
    let mut c = typing("Hi");
    assert_eq!(c.end(), Did::Change);
    assert!(!c.typing());
    let view = c.view();
    let hi = &view.drawings[0];
    assert_eq!(hi.mark, words(pt(30, 40), "Hi", false));
    assert_eq!(
        (hi.ink, hi.size),
        (view.ink, view.size),
        "the ink and text size picked"
    );

    c.press(pt(10, 10), SAME);
    c.write("Yo");
    assert_eq!(
        c.press(pt(80, 80), SAME),
        Did::Change,
        "a click elsewhere ends it"
    );
    assert!(!c.typing(), "and opens no new text");
    assert_eq!(drawn(&c).len(), 2);
    assert_eq!(c.end(), Did::Stay, "no text is open");
}

#[test]
fn an_empty_text_is_not_added() {
    for text in ["", "   ", "\u{200f}"] {
        let mut c = typing(text);
        assert_eq!(c.end(), Did::Show, "{text:?}");
        assert!(drawn(&c).is_empty());
        assert!(!c.typing());
    }
}

#[test]
fn arabic_text_is_kept_as_typed_and_reads_right_to_left() {
    let arabic = "مَرْحَبًا بِكُم";
    let mut c = with(Tool::Text);
    c.press(pt(100, 30), SAME);
    for letter in arabic.chars() {
        c.write(&letter.to_string());
    }
    let units = u32::try_from(arabic.encode_utf16().count()).unwrap();
    assert_eq!(typed(&c), Some((pt(100, 30), arabic.into(), units, true)));
    c.end();
    assert_eq!(marks(&c), [words(pt(100, 30), arabic, true)]);

    let mut c = typing("بِ");
    c.erase();
    assert_eq!(
        typed(&c).map(|t| t.1),
        Some("ب".into()),
        "Backspace takes the mark alone"
    );
    for (text, rtl) in [("Hi مرحبا", false), ("12 مرحبا", true)] {
        let c = typing(text);
        assert_eq!(
            typed(&c).map(|t| t.3),
            Some(rtl),
            "the first letter sets {text:?}"
        );
    }
}

#[test]
fn ink_and_size_restyle_the_text_being_typed() {
    let mut c = typing("Hi");
    assert_eq!(c.ink(Ink::Green), Did::Show);
    assert_eq!(c.size(true), Did::Show);
    assert!(c.typing(), "they keep it open");
    let size = c.view().size;
    c.end();
    let hi = &drawn(&c)[0];
    assert_eq!((hi.ink, hi.size), (Ink::Green, size));
}

#[test]
fn undo_while_typing_takes_the_text_away_and_redo_brings_it_back() {
    let mut c = typing("Hi");
    assert!(c.view().undo);
    assert_eq!(c.undo(), Did::Show, "the picture is as before the text");
    assert!(!c.typing());
    assert!(drawn(&c).is_empty());
    assert_eq!(c.redo(), Did::Change);
    assert_eq!(marks(&c), [words(pt(30, 40), "Hi", false)]);
}

#[test]
fn picking_a_tool_ends_the_text() {
    let mut c = typing("Hi");
    assert_eq!(c.tool(Tool::Pen), Did::Change);
    assert!(!c.typing());
    assert_eq!(marks(&c), [words(pt(30, 40), "Hi", false)]);
}

#[test]
fn redo_is_not_offered_while_a_text_is_typed_and_its_step_waits() {
    let mut c = with(Tool::Steps);
    c.press(pt(10, 10), SAME);
    c.undo();
    c.tool(Tool::Text);
    c.press(pt(30, 40), SAME);
    assert!(c.view().redo, "an empty text takes nothing from Redo");
    c.write("Hi");
    assert!(!c.view().redo, "a typed text would replace Redo's step");
    assert_eq!(c.redo(), Did::Stay, "so a stray Redo does nothing");
    assert!(c.typing(), "and the text stays open");
    c.erase();
    c.erase();
    assert_eq!(
        c.redo(),
        Did::Change,
        "an emptied text closes, and Redo acts"
    );
    assert!(!c.typing());
    assert_eq!(
        marks(&c),
        [Mark::Step {
            at: pt(10, 10),
            n: 1
        }]
    );
}

#[test]
fn undo_while_typing_with_undo_full_keeps_every_step() {
    let max = *UNDO_MAX.start();
    let mut c = with(Tool::Steps);
    for x in 1..=i32::from(max) {
        c.press(pt(x, 5), SAME);
    }
    c.tool(Tool::Text);
    c.press(pt(30, 40), SAME);
    c.write("Hi");
    assert_eq!(c.undo(), Did::Show);
    let mut undone = 0;
    for _ in 0..=max {
        if c.undo() == Did::Change {
            undone += 1;
        }
    }
    assert_eq!(
        undone, max,
        "every step before the text can still be taken back"
    );
    assert!(drawn(&c).is_empty());
}

#[test]
fn the_line_takes_its_keys_only_while_text_is_typed() {
    let mut c = with(Tool::Text);
    assert_eq!(c.line(&PanelKey::Esc), None, "Esc goes on to the flow");
    assert_eq!(c.line(&letters("a")), None);
    c.press(pt(10, 10), SAME);
    let keys = [
        (letters("ab"), "ab", 2),
        (PanelKey::Tick, "ab ", 3),
        (PanelKey::Erase, "ab", 2),
        (PanelKey::Left, "ab", 1),
        (PanelKey::Delete, "a", 1),
        (
            PanelKey::Word {
                erase: 1,
                text: Redacted::new("xy".to_owned()),
            },
            "xy",
            2,
        ),
    ];
    for (key, text, caret) in keys {
        assert_eq!(c.line(&key), Some(Did::Show), "{key:?}");
        assert_eq!(typed(&c).map(|t| (t.1, t.2)), Some((text.into(), caret)));
    }
    assert_eq!(c.line(&PanelKey::Up), Some(Did::Stay));
    assert_eq!(
        c.line(&PanelKey::Esc),
        Some(Did::Change),
        "Esc ends it, keeping it"
    );
    c.press(pt(50, 50), SAME);
    c.write("z");
    assert_eq!(c.line(&PanelKey::Enter), Some(Did::Change));
    assert_eq!(drawn(&c).len(), 2);
}

#[test]
fn typed_text_never_prints() {
    let mut c = typing("secret words");
    let open = format!("{c:?} {:?}", c.view());
    c.end();
    let ended = format!("{c:?} {:?}", c.view());
    for printed in [open, ended] {
        assert!(!printed.contains("secret"), "{printed}");
    }
}
