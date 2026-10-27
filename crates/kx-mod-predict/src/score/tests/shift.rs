//! A latched Shift: the pick spends it, so every chip shows its first capital.

use super::*;

#[test]
fn a_latched_shift_capitalises_the_first_letter_before_or_after_typed_letters() {
    // The pick spends the Shift, so the chip must show what it did.
    let en = english();
    let shift = TypingMode {
        shift: true,
        ..TypingMode::default()
    };
    let next = run(&en, &NoLearned, &ask(&["in"], ""), shift);
    assert_eq!(next.first().map(String::as_str), Some("The"));
    let typed = run(&en, &NoLearned, &ask(&[], "th"), shift);
    assert_eq!(typed.first().map(String::as_str), Some("The"));
    let plain = run(&en, &NoLearned, &ask(&[], "th"), TypingMode::default());
    assert_eq!(
        plain.first().map(String::as_str),
        Some("the"),
        "no Shift, no capital"
    );
}
