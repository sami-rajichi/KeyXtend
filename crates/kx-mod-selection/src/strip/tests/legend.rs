//! The legend: each strip shows the one its settings give, and a new one draws a new strip, so a click on the old
//! strip misses.

use super::*;

/// The round and legend of a view that shows a strip; it fails the test otherwise.
fn legend_of(view: Option<&ChipsView>) -> (u32, Legend) {
    match view {
        Some(&ChipsView::Shown { round, legend, .. }) => (round, legend),
        other => panic!("no strip: {other:?}"),
    }
}

#[test]
fn a_strip_shows_the_legend_its_settings_give_and_a_new_one_redraws_it() {
    let words = StripConfig {
        look: Legend::Words,
        ..cfg()
    };
    let mut rig = Rig::with(words);
    let shown = rig.step(look(sel(1, SelKind::Editable, 40)));
    let (round, legend) = legend_of(shown.as_ref());
    assert_eq!(legend, Legend::Words);
    let icons = StripConfig {
        look: Legend::Icons,
        ..cfg()
    };
    let redrawn = rig.strip.configure(icons, rig.now);
    let (again, legend) = legend_of(redrawn.as_ref());
    assert_eq!(legend, Legend::Icons);
    assert!(again > round, "a new strip, so an old click misses");
    assert_eq!(rig.strip.configure(icons, rig.now), None, "the same again");
}
