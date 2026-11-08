//! The legend: each strip shows the one the settings the module started with give.

use super::*;

/// The legend of a view that shows a strip; it fails the test otherwise.
fn legend_of(view: Option<&ChipsView>) -> Legend {
    match view {
        Some(&ChipsView::Shown { legend, .. }) => legend,
        other => panic!("no strip: {other:?}"),
    }
}

#[test]
fn a_strip_shows_the_legend_its_settings_give() {
    for want in Legend::ALL {
        let mut rig = Rig::with(StripConfig {
            look: want,
            ..cfg()
        });
        let shown = rig.step(look(sel(1, SelKind::Editable, 40)));
        assert_eq!(legend_of(shown.as_ref()), want);
        let caret = rig.step(click(caret(2, 50)));
        assert_eq!(legend_of(caret.as_ref()), want, "the caret's Paste too");
    }
}
