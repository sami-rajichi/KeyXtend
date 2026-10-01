//! What only the app sees, since it alone depends on every crate: the board, the modules and the language key's
//! designs have their texts in every language, the board's icons resolve, every paint names tokens each theme has, and
//! every character key has its scan code in the layouts table.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

#[path = "../common/fakes.rs"]
mod fakes;
mod icons;
mod paint;
mod scans;
mod texts;

use kx_mod_keyboard::view::{BoardView, board_view};
use kx_mod_keyboard::{BoardSpec, KeyboardSettings, LangKey};

/// Every design of the language key.
fn designs() -> [LangKey; 3] {
    // Exhaustive on purpose: a new design stops this building until it is listed here.
    let listed = |d: LangKey| match d {
        LangKey::Carousel | LangKey::Cycle | LangKey::List => d,
    };
    [LangKey::Carousel, LangKey::Cycle, LangKey::List].map(listed)
}

/// The shipped board at its default size, with the language key in `design`.
fn board(design: LangKey) -> BoardView {
    let spec = BoardSpec::shipped().unwrap();
    let mut settings: KeyboardSettings = kx_mod_keyboard::SPEC.defaults_as().unwrap();
    settings.lang_key = design;
    board_view(&spec, &settings, settings.size)
}
