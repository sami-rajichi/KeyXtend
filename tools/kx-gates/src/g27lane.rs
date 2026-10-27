//! The product's chip lane as UI Automation shows it on the keyboard window: its chips by gate id, the lock note, and
//! the word in a chip's name, which never reaches a result line.
#![cfg(windows)]

use std::fmt;

use kx_mod_predict::{Casing, fold};
use kx_module_api::Redacted;
use windows::Win32::Foundation::{HWND, RECT};

use crate::keymap::{KeyRead, Reader};
use crate::p7cfg::G27;

/// What the lane shows. The best chip's name holds a word, so it stays redacted and `Debug` shows counts only.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Lane {
    /// The lock note shows.
    pub locked: bool,
    /// Chips in UI Automation's tree, shown or not.
    pub chips: usize,
    /// Chips with a box on screen.
    pub shown: usize,
    /// The best chip's name, when it shows.
    pub first: Option<Redacted<String>>,
}

impl fmt::Debug for Lane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Lane")
            .field("locked", &self.locked)
            .field("chips", &self.chips)
            .field("shown", &self.shown)
            .field("first", &self.first.is_some())
            .finish()
    }
}

/// True when box `r` has an area on screen; a hidden item's box is empty.
fn on_screen(r: &RECT) -> bool {
    r.right > r.left && r.bottom > r.top
}

/// The lane among `buttons`, with the lock note's box `locked`: chips are the buttons whose id starts with `start`,
/// and the best one has the id `best`.
pub fn lane_of(buttons: &[KeyRead], locked: Option<RECT>, start: &str, best: &str) -> Lane {
    let chips: Vec<&KeyRead> = buttons.iter().filter(|b| b.id.starts_with(start)).collect();
    let first = chips.iter().find(|b| b.id == best && on_screen(&b.rect));
    Lane {
        locked: locked.is_some_and(|r| on_screen(&r)),
        chips: chips.len(),
        shown: chips.iter().filter(|b| on_screen(&b.rect)).count(),
        first: first.map(|b| Redacted::new(b.name.clone())),
    }
}

/// The lane on keyboard window `hwnd` now.
pub fn read(reader: &Reader, hwnd: HWND, g: &G27) -> Result<Lane, String> {
    let buttons = reader.walk(hwnd)?;
    let locked = reader.rect(hwnd, &g.locked_id()).ok();
    Ok(lane_of(&buttons, locked, &g.chip_start(), &g.chip_id(0)))
}

/// The word in chip name `name`: what lies between the text before and after it in the first of `forms` that fits.
pub fn word_of(name: &str, forms: &[(String, String)]) -> Option<Redacted<String>> {
    forms.iter().find_map(|(before, after)| {
        let word = name
            .strip_prefix(before.as_str())?
            .strip_suffix(after.as_str())?;
        (!word.is_empty()).then(|| Redacted::new(word.to_string()))
    })
}

/// True when `word` offers `prefix` in language `lang`: its folded form starts with the prefix's, as the engine
/// matches them.
pub fn offers(word: &str, prefix: &str, lang: &str) -> bool {
    let casing = Casing::of(lang);
    fold(word, casing).starts_with(&fold(prefix, casing))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A box on screen, and an empty one.
    const SHOWN: RECT = RECT {
        left: 10,
        top: 10,
        right: 60,
        bottom: 30,
    };
    const HIDDEN: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    fn button(id: &str, name: &str, rect: RECT) -> KeyRead {
        KeyRead {
            id: id.into(),
            name: name.into(),
            rect,
            invokable: true,
        }
    }

    fn lane(buttons: &[KeyRead], locked: Option<RECT>) -> Lane {
        lane_of(buttons, locked, "words.chip-", "words.chip-0")
    }

    /// The text a redacted value holds, for the checks.
    fn text(r: Option<&Redacted<String>>) -> Option<&str> {
        r.map(|t| t.expose().as_str())
    }

    #[test]
    fn a_lanes_debug_shows_counts_and_never_a_word() {
        let got = lane(&[button("words.chip-0", "Suggestion: secret", SHOWN)], None);
        let shown = format!("{got:?}");
        assert!(!shown.contains("secret"), "{shown}");
        assert!(shown.contains("chips: 1"), "{shown}");
    }

    #[test]
    fn chips_are_counted_by_id_and_the_best_names_the_shown_first_chip() {
        let read = [
            button("bar.grip", "Move", SHOWN),
            button("words.chip-0", "Suggestion: hello", SHOWN),
            button("words.chip-1", "Suggestion: help", SHOWN),
            button("words.chip-2", "Suggestion: helmet", HIDDEN),
            button("words.menu", "Suggestions menu", SHOWN),
        ];
        let got = lane(&read, None);
        assert_eq!((got.chips, got.shown, got.locked), (3, 2, false));
        assert_eq!(text(got.first.as_ref()), Some("Suggestion: hello"));
    }

    #[test]
    fn the_lock_shows_only_with_a_box_and_a_hidden_first_chip_is_no_best() {
        let read = [button("words.chip-0", "Suggestion: x", HIDDEN)];
        let got = lane(&read, Some(SHOWN));
        assert!(got.locked && got.first.is_none());
        assert_eq!((got.chips, got.shown), (1, 0), "in the tree, not shown");
        assert!(!lane(&[], Some(HIDDEN)).locked, "a hidden note");
        assert!(!lane(&[], None).locked, "no note");
    }

    #[test]
    fn a_chips_word_comes_out_of_its_name_in_any_shipped_language() {
        let g = crate::config::load().expect("loads").g27;
        let forms = crate::ftl::around(&g.chip_name, &g.chip_word).expect("in all three");
        let name = |i: usize, w: &str| format!("{}{w}{}", forms[i].0, forms[i].1);
        assert_eq!(
            text(word_of(&name(0, "hello"), &forms).as_ref()),
            Some("hello")
        );
        assert_eq!(
            text(word_of(&name(1, "école"), &forms).as_ref()),
            Some("école")
        );
        assert_eq!(text(word_of(&name(2, "أن"), &forms).as_ref()), Some("أن"));
        assert_eq!(word_of("Undo", &forms), None, "Undo holds no word");
        assert_eq!(word_of(&name(0, ""), &forms), None, "no word at all");
    }

    #[test]
    fn a_word_offers_a_prefix_when_their_folded_forms_start_alike() {
        assert!(offers("hello", "hel", "en"));
        assert!(offers("Hello", "hel", "en"), "capitals fold");
        assert!(offers("école", "eco", "fr"), "accents fold");
        assert!(offers("أن", "ان", "ar"), "hamza on alef folds");
        assert!(!offers("world", "hel", "en"));
        assert!(!offers("he", "hel", "en"), "shorter than the prefix");
    }
}
