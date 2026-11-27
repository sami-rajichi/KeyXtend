//! The frozen screens never trap the user (ADR-0059): each time one of our windows shows or Snip's windows change while
//! frozen screens show, the hub asks for them to lie under every other window of ours, never while a system dialog sits
//! in front or no screen is frozen; and guard runs spare Snip's windows, so the dialog stays above them.

use super::{editing, heard, idle, page, picking, screens, strip};
use crate::Rig;
use crate::logs::warnings_of;
use kx_module_api::ports::{Countdown, Part, SnipPage, SnipStage};
use kx_platform::ui::UiError;
use kx_platform_fake::GuardCall;
use kx_ui::i18n::keys::{SNIP_WINDOWS, WINDOW_KEYBOARD, WINDOW_SNIP_PANEL, WINDOW_SNIP_SCREEN};

/// The frozen screens' title after a language switch, as QML gives it.
const RETITLED: &str = "KeyXtend test: frozen screens, retitled";

/// The guard's calls while the Qt thread orders the windows once, by the frozen screens' title.
fn ordered(rig: &Rig) -> Vec<GuardCall> {
    let from = rig.guard.calls().len();
    rig.hub.snip_order(&screen(rig));
    rig.guard.calls().split_off(from)
}

/// The frozen screens' title, in the keyboard's language.
fn screen(rig: &Rig) -> String {
    rig.hub.i18n().text(WINDOW_SNIP_SCREEN)
}

/// Round `round`'s strip, with a system dialog in front.
fn dialog(round: u32) -> SnipPage {
    SnipPage {
        dialog: true,
        ..strip(round)
    }
}

/// The titles every guard run spares: Snip's windows and its panel, in the keyboard's language.
pub(crate) fn spared(rig: &Rig) -> Vec<String> {
    let keys = SNIP_WINDOWS.into_iter().chain([WINDOW_SNIP_PANEL]);
    keys.map(|k| rig.hub.i18n().text(k)).collect()
}

/// While a system dialog shows over frozen screens, a guard run, as the bubble tap or the tray menu makes, lifts no
/// window of Snip's: it spares them all, the frozen screens first, so they stay out of the band under the dialog.
#[test]
fn a_guard_run_over_a_dialog_spares_every_snip_window_and_lays_nothing() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, dialog(1));
    let from = rig.guard.calls().len();
    rig.hub.guard(1, Some("menu"));
    let calls = rig.guard.calls().split_off(from);
    let runs: Vec<_> = calls
        .iter()
        .filter(|c| matches!(c, GuardCall::GuardOwn(..)))
        .collect();
    assert_eq!(
        runs,
        [&GuardCall::GuardOwn(spared(&rig), Some("menu".into()))]
    );
    assert!(spared(&rig).contains(&screen(&rig)), "the frozen screens");
    let keyboard = rig.hub.i18n().text(WINDOW_KEYBOARD);
    assert!(
        !spared(&rig).contains(&keyboard),
        "the keyboard stays guarded"
    );
    assert!(!calls.iter().any(|c| matches!(c, GuardCall::LayLow(_))));
}

/// Once the dialog closes, the next view lays the frozen screens under our windows again.
#[test]
fn the_order_comes_back_once_the_dialog_closes() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, dialog(1));
    assert!(ordered(&rig).is_empty(), "the dialog is in front");
    heard.shown(&rig, strip(2));
    assert_eq!(ordered(&rig), [GuardCall::LayLow(screen(&rig))]);
}

#[test]
fn frozen_screens_go_under_our_windows_while_picking_and_on_the_strip() {
    let rig = Rig::new();
    let heard = heard(&rig);
    for shown in [picking(1, screens()), strip(2)] {
        heard.shown(&rig, shown);
        assert_eq!(ordered(&rig), [GuardCall::LayLow(screen(&rig))]);
    }
}

#[test]
fn nothing_is_laid_with_no_frozen_screen_or_a_dialog_in_front() {
    let rig = Rig::new();
    assert!(ordered(&rig).is_empty(), "before any view");
    let heard = heard(&rig);
    let part = Part::new(super::rect(1, 1, 4, 3)).unwrap();
    let count = SnipStage::Count(Countdown { part, left_s: 3 });
    for shown in [
        idle(1),
        page(2, Vec::new(), count),
        editing(3, None),
        dialog(4),
    ] {
        heard.shown(&rig, shown);
        assert!(ordered(&rig).is_empty());
    }
}

#[test]
fn a_refusal_warns_once_per_snip_and_frozen_screens_not_made_yet_are_quiet() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let title = screen(&rig);
    heard.shown(&rig, picking(1, screens()));
    rig.guard.set_error(Some(UiError::NoWindow));
    assert!(warnings_of(|| rig.hub.snip_order(&title)).is_empty());
    rig.guard.set_error(Some(UiError::Refused(5)));
    let twice = || {
        rig.hub.snip_order(&title);
        rig.hub.snip_lay();
    };
    assert_eq!(warnings_of(twice).len(), 1, "once for these frozen screens");
    heard.shown(&rig, strip(2));
    assert!(
        warnings_of(|| rig.hub.snip_lay()).is_empty(),
        "the same snip"
    );
    heard.shown(&rig, idle(3));
    heard.shown(&rig, picking(4, screens()));
    assert_eq!(warnings_of(|| rig.hub.snip_lay()).len(), 1, "the next snip");
}

/// A window of ours that shows lays the frozen screens by the title QML gave last (`Snip.order`), none before.
#[test]
fn a_window_shown_lays_them_by_the_title_qml_gave_last() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, picking(1, screens()));
    let shown = || {
        let from = rig.guard.calls().len();
        rig.hub.snip_lay();
        rig.guard.calls().split_off(from)
    };
    assert!(shown().is_empty(), "no title yet");
    rig.hub.snip_order(RETITLED);
    assert_eq!(shown(), [GuardCall::LayLow(RETITLED.into())]);
}
