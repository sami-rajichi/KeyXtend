//! Our windows beside a capture: a capture window is made hidden by its title before its shield, after its owner, which
//! a window made ahead stays over, and while the keyboard is aside a window of ours that Qt shows is kept out of every
//! capture before the screen shows it, but not once it is back; while frozen screens show, such a window lays them
//! under it first.

use kx_module_api::Bus;
use kx_module_api::ports::{
    BASE_DPI, FrozenScreen, Rect, Shot, SnipAside, SnipPage, SnipStage, SnipView,
};
use kx_platform_fake::GuardCall;
use kx_test_support::host::FakeCore;
use kx_ui::i18n::keys::{WINDOW_CLIP, WINDOW_SNIP_SCREEN};
use std::sync::Arc;

/// Shows a window titled `title` under the show watch; each shield of that title meanwhile, by whether it was on.
fn shown(title: &str) -> Result<Vec<bool>, String> {
    let guard = crate::guard();
    let from = guard.calls().len();
    if !kx_ui::show_watched(title) {
        return Err(format!(
            "{title} was not watched after Qt made it and before it showed"
        ));
    }
    let calls = guard.calls().split_off(from);
    let on = |c: &GuardCall| match c {
        GuardCall::Shield(t, on) if t == title => Some(*on),
        _ => None,
    };
    Ok(calls.iter().filter_map(on).collect())
}

/// A view at rest over `frozen` screens.
fn view(frozen: Vec<FrozenScreen>) -> SnipView {
    SnipView::new(SnipPage {
        round: 1,
        frozen,
        stage: SnipStage::Idle,
        ask: None,
        note: None,
        panel: None,
        dialog: false,
    })
}

/// A capture window Qt has not made yet is made by its title, counted and left hidden, so its shield comes first.
pub fn a_titled_window_is_made_hidden_before_its_shield() -> Result<(), String> {
    let title = "kx tint titled";
    if !kx_ui::titled_made(title) {
        return Err(format!("{title} was not made alone, hidden, by its title"));
    }
    Ok(())
}

/// A shadow, its window and that window's tip sharing a title are made each after its owner, so the window starts in
/// front of its shadow, though Qt lists the newest window first.
pub fn a_titled_shadow_is_made_before_the_window_it_owns() -> Result<(), String> {
    let title = "kx tint owned";
    if !kx_ui::owners_made_first(title) {
        return Err(format!(
            "{title}'s windows were not each made after their owner, hidden"
        ));
    }
    Ok(())
}

/// A window made ahead with its shadow and shown as Glow shows it stays over that shadow, owned by it, through a system
/// dialog, a maximise, a restore and a tip. A window made at its show is only told, as its first broken step names why
/// the panel's shadow lay over it.
pub fn a_made_ahead_window_stays_over_its_shadow() -> Result<(), String> {
    let lazy = kx_ui::glow_pair_broken("kx tint lazy", "kx tint shade", false);
    eprintln!("a shadowed window made at its show first broke at step {lazy} (0: never)");
    match kx_ui::glow_pair_broken("kx tint glow", "kx tint shade", true) {
        0 => Ok(()),
        u8::MAX => Err("making the window ahead by its title missed it or its shadow".to_owned()),
        step => Err(format!(
            "a window made ahead fell under its shadow or lost it as owner at step {step}"
        )),
    }
}

/// A window shown while frozen screens show lays them under it and our other windows before it shows (ADR-0059).
pub fn a_window_shown_over_frozen_screens_lays_them_under_it_first() -> Result<(), String> {
    let hub = crate::hub()?;
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _heard = hub.listen(&bus).map_err(|e| e.to_string())?;
    let rect = Rect {
        x: 0,
        y: 0,
        w: 2,
        h: 2,
    };
    let shot = Shot::new(2, 2, vec![0; 16]).ok_or("a 2 by 2 shot did not make")?;
    let screen = FrozenScreen::new(rect, BASE_DPI, shot).ok_or("a frozen screen did not make")?;
    bus.publish(&view(vec![screen]))
        .map_err(|e| e.to_string())?;
    let screens = hub.i18n().text(WINDOW_SNIP_SCREEN);
    hub.snip_order(&screens);
    let guard = crate::guard();
    let from = guard.calls().len();
    let title = hub.i18n().text(WINDOW_CLIP);
    let early = kx_ui::show_watched(&title);
    let during = guard.calls().split_off(from);
    bus.publish(&view(Vec::new())).map_err(|e| e.to_string())?;
    if !early {
        return Err(format!("{title} was not watched while made and hidden"));
    }
    if during != [GuardCall::LayLow(screens)] {
        return Err(format!(
            "{title} showed before the frozen screens went under it: {during:?}"
        ));
    }
    Ok(())
}

/// A clipboard panel shown during an aside is shielded as it shows; one shown with the keyboard back is not.
pub fn a_window_shown_during_an_aside_is_shielded_before_it_shows() -> Result<(), String> {
    let hub = crate::hub()?;
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _heard = hub.listen(&bus).map_err(|e| e.to_string())?;
    let title = hub.i18n().text(WINDOW_CLIP);
    bus.publish(&SnipAside::Bubble).map_err(|e| e.to_string())?;
    let during = shown(&title)?;
    bus.publish(&SnipAside::Back).map_err(|e| e.to_string())?;
    let after = shown(&title)?;
    if during != [true] || !after.is_empty() {
        return Err(format!(
            "{title} was shielded {during:?} during the aside and {after:?} after it"
        ));
    }
    Ok(())
}
