//! Tests of `g25`.

use kx_mod_clipboard::{ClipSettings, ID, SPEC};
use kx_settings::peek_section;
use toml::Table;

use super::*;
use crate::clipcopy::clip_table;
use crate::copyend::Gone;
use crate::winpanel::{NO_UI_ACCESS, SKIPPED};

#[test]
fn the_click_lands_in_the_middle_of_the_part_that_steps_on() {
    let key = RECT {
        left: 100,
        top: 10,
        right: 400,
        bottom: 40,
    };
    let share = 0.3333;
    let p = forward(&key, share);
    assert_eq!((p.x, p.y), (300, 25));
    assert!(
        f64::from(p.x) > 100.0 + 300.0 * f64::from(share),
        "past the back part"
    );
    assert_eq!(forward(&key, 0.0).x, 250, "no back part: the middle");
}

#[test]
fn only_the_list_design_is_skipped_since_a_click_opens_the_list() {
    assert_eq!(skip_reason(LangKey::Carousel), None);
    assert_eq!(skip_reason(LangKey::Cycle), None);
    assert!(skip_reason(LangKey::List).is_some());
}

#[test]
fn g25s_own_copy_runs_with_the_history_key_off() {
    let c = crate::config::load().expect("loads");
    let t = clip_table(&Table::new(), c.g25.history_key, &c.g20_product.skip_app);
    let s: ClipSettings = peek_section(Some(&t.expect("built")), ID, &SPEC).expect("reads back");
    assert!(!s.history_key, "Win and V reach Windows' own history");
    assert!(c.g25.ready_ms > 0);
}

#[test]
fn g25_passes_only_when_its_parts_its_copy_and_the_settings_did() {
    let quit = Ended {
        gone: Gone::Quit,
        note: None,
        released: Vec::new(),
    };
    let good = || Ok(json!({ "pass": true }));
    assert_eq!(on_copy(good(), Some(&quit), &Ok(()))["pass"], true);
    assert_eq!(on_copy(good(), None, &Ok(()))["pass"], false, "no copy");
    let back = on_copy(good(), Some(&quit), &Err("locked".into()));
    assert_eq!(
        (back["pass"].as_bool(), back["settings_error"].as_str()),
        (Some(false), Some("locked"))
    );
    let failed = on_copy(Err("no keyboard".into()), Some(&quit), &Ok(()));
    assert_eq!(
        (failed["pass"].as_bool(), failed["error"].as_str()),
        (Some(false), Some("no keyboard"))
    );
    let crashed = Ended {
        gone: Gone::Crashed(Some(1)),
        ..quit
    };
    assert_eq!(on_copy(good(), Some(&crashed), &Ok(()))["pass"], false);
}

#[test]
fn lang_is_the_low_word_in_hex() {
    assert_eq!(lang(0x0409_0409), "0409");
    assert_eq!(
        lang(isize::try_from(0xF002_1C01_u32).expect("fits")),
        "1C01"
    );
    let extended = isize::try_from(0xFFFF_FFFF_F029_1C01_u64.cast_signed()).expect("64-bit");
    assert_eq!(lang(extended), "1C01", "a sign-extended handle");
}

/// A copy's product token, with uiAccess or as the test build gives it.
fn product(ui_access: bool) -> Rights {
    Rights {
        ui_access,
        level: workertok::MEDIUM,
    }
}

fn ok(yes: bool) -> Value {
    json!({ "ok": yes })
}

/// G25's result with every other part passed and `window` as its Win and V part.
fn with_window(window: Value) -> Value {
    summed(
        AppKind::Notepad.name(),
        [ok(true), ok(true), ok(true), window, ok(true)],
    )
}

#[test]
fn a_product_without_ui_access_skips_win_and_v_and_g25_can_still_pass() {
    let window = history_off(Some(product(false)), || {
        panic!("Win and V ran on a product without uiAccess")
    });
    let v = with_window(window.clone());
    assert_eq!(v["clipboard_window"][SKIPPED], true, "{v}");
    assert_eq!(v["clipboard_window"]["why"], NO_UI_ACCESS);
    assert_eq!(v["pass"], true, "a skipped part fails nothing");
    let others_failed = [ok(false), ok(true), ok(true), window, ok(true)];
    let failed = summed(AppKind::Notepad.name(), others_failed);
    assert_eq!(failed["pass"], false, "nor passes anything");
}

#[test]
fn a_ui_access_product_still_runs_win_and_v_and_judges_it() {
    let failed = history_off(Some(product(true)), || ok(false));
    assert_eq!(with_window(failed)["pass"], false);
    let good = history_off(Some(product(true)), || ok(true));
    assert_eq!(with_window(good)["pass"], true);
    let unread = history_off(None, || ok(false));
    assert_eq!(
        with_window(unread)["pass"],
        false,
        "an unread token still runs it"
    );
}
