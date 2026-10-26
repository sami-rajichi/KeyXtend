//! What only the app sees, since it alone depends on every crate: the board, the shortcuts layer, the modules and the
//! language key's designs have their texts in every language, the shortcuts agree with the Windows table, the board's
//! icons resolve, every paint names tokens each theme has, the Next key holds the most pages' dots, every character
//! key has its scan code in the layouts table, a size, a buffer pool or the shortcut pages past a rule go back to
//! their default, the stage puts Qt's QML where the app pins it, and the longest word with chips fits the keyboard's
//! text cap.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

mod dots;
#[path = "../common/fakes.rs"]
mod fakes;
mod icons;
mod pages;
mod paint;
mod pool;
mod scans;
// The Windows adapter serves its shortcut table on Windows only.
#[cfg(windows)]
mod shortcuts;
mod sizes;
mod stage;
mod texts;
// Predict is an optional module.
#[cfg(feature = "mod-predict")]
mod words;

use kx_mod_keyboard::view::{BoardView, board_view, with_layer};
use kx_mod_keyboard::{BoardSpec, KeyboardSettings, LangKey, Layer, SPEC};
use kx_module_api::ports::ShortcutId;
use std::collections::HashSet;

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
    let mut settings: KeyboardSettings = SPEC.defaults_as().unwrap();
    settings.lang_key = design;
    board_view(&spec, &settings, settings.size)
}

/// The owner's shortcut pages, the default.
fn pages() -> Vec<Vec<ShortcutId>> {
    SPEC.defaults_as::<KeyboardSettings>()
        .unwrap()
        .shortcut_pages
}

/// The board with the language key in `design`: the layer hidden, then shown on each of its pages.
fn views(design: LangKey) -> Vec<BoardView> {
    let spec = BoardSpec::shipped().unwrap();
    let pages = pages();
    let known: HashSet<ShortcutId> = pages.iter().flatten().cloned().collect();
    let mut layer = Layer::new(spec.layer_grid(), &pages, &known);
    let mut views = vec![board(design)];
    layer.toggle();
    for _ in &pages {
        let mut view = board(design);
        with_layer(&mut view, &spec, &layer, &[]);
        views.push(view);
        layer.next_page();
    }
    views
}
