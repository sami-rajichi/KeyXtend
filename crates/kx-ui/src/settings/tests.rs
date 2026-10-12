use super::*;
use kx_module_api::parse_as;
use kx_theme::contrast::{MAX_CONTRAST, WCAG_AA};
use toml::{Table, Value};

/// The defaults with `key` set to `value`.
fn with(key: &str, value: impl Into<Value>) -> Table {
    let mut t: Table = SPEC.defaults.parse().unwrap();
    t.insert(key.to_owned(), value.into());
    t
}

/// True when `t` parses but breaks a rule.
fn invalid(t: &Table) -> bool {
    matches!(validate_as::<UiSettings>(t), Err(SettingsError::Invalid(_)))
}

/// True when `t` does not fit the settings type.
fn unparsed(t: &Table) -> bool {
    matches!(validate_as::<UiSettings>(t), Err(SettingsError::Parse(_)))
}

#[test]
fn the_defaults_are_the_plan_v1_values() {
    let s = UiSettings::from_defaults().unwrap();
    assert_eq!((s.theme.as_str(), s.mode), ("native", ModeChoice::Auto));
    assert!(!s.follow_accent && !s.reduced_motion);
    assert_eq!((s.backdrop, s.language.as_str()), (Backdrop::None, AUTO));
    assert_eq!((s.tip_ms, s.poll_ms, s.guard_delay_ms), (500, 16, 300));
    assert_eq!(s.tray_grace_ms, 500);
    assert_eq!((s.guard_tries, s.page_rows), (10, 5));
    assert_eq!((s.send_tries, s.send_wait_ms), (10, 100));
    assert_eq!((s.device_rebuilds, s.device_retry_ms), (3, 500));
    assert!((s.min_contrast - WCAG_AA).abs() < f32::EPSILON);
    assert_eq!(SPEC.version, 1);
    assert!(SPEC.is_consistent());
}

#[test]
fn a_theme_that_kx_theme_lacks_is_refused() {
    let Err(SettingsError::Invalid(why)) = validate_as::<UiSettings>(&with("theme", "natve"))
    else {
        panic!("a theme typo must be refused");
    };
    assert!(
        why.contains("native, et66, dolch"),
        "the message names the themes: {why}"
    );
    for id in ["native", "et66", "dolch"] {
        assert_eq!(
            validate_as::<UiSettings>(&with("theme", id)),
            Ok(()),
            "{id}"
        );
    }
}

#[test]
fn a_language_is_auto_or_one_the_keyboard_speaks() {
    for tag in ["auto", "en", "fr", "ar"] {
        assert_eq!(
            validate_as::<UiSettings>(&with("language", tag)),
            Ok(()),
            "{tag}"
        );
    }
    assert!(invalid(&with("language", "de")));
    assert!(invalid(&with("language", "")));
    assert!(invalid(&with("language", "AR")), "tags match exactly");
}

#[test]
fn min_contrast_never_goes_below_wcag_aa() {
    assert!(invalid(&with("min_contrast", 4.4)));
    assert!(invalid(&with("min_contrast", 1.0)));
    assert!(invalid(&with(
        "min_contrast",
        f64::from(MAX_CONTRAST) + 1.0
    )));
    assert!(invalid(&with("min_contrast", f64::NAN)), "NaN is refused");
    assert_eq!(
        validate_as::<UiSettings>(&with("min_contrast", 7.0)),
        Ok(())
    );
    assert_eq!(
        validate_as::<UiSettings>(&with("min_contrast", f64::from(MAX_CONTRAST))),
        Ok(())
    );
}

#[test]
fn zero_timings_and_rows_are_refused() {
    for key in [
        "tip_ms",
        "poll_ms",
        "guard_delay_ms",
        "guard_tries",
        "send_tries",
        "send_wait_ms",
        "page_rows",
    ] {
        assert!(invalid(&with(key, 0)), "{key}");
        assert!(unparsed(&with(key, -1)), "{key}");
    }
}

#[test]
fn timings_and_rows_past_their_range_are_refused() {
    let past = |range_end: u64| i64::try_from(range_end).unwrap() + 1;
    assert!(invalid(&with("tip_ms", past(*TIP_MS.end()))));
    assert!(invalid(&with("poll_ms", past(*POLL_MS.end()))));
    assert!(invalid(&with(
        "guard_delay_ms",
        past(*GUARD_DELAY_MS.end())
    )));
    assert!(invalid(&with(
        "guard_tries",
        past(u64::from(*GUARD_TRIES.end()))
    )));
    assert!(invalid(&with(
        "send_tries",
        past(u64::from(*SEND_TRIES.end()))
    )));
    assert!(invalid(&with("send_wait_ms", past(*SEND_WAIT_MS.end()))));
    let rows = u64::try_from(*PAGE_ROWS.end()).unwrap();
    assert!(invalid(&with("page_rows", past(rows))));
    let top = i64::try_from(*TIP_MS.end()).unwrap();
    assert_eq!(validate_as::<UiSettings>(&with("tip_ms", top)), Ok(()));
}

#[test]
fn the_tray_grace_may_be_off_but_not_past_its_range() {
    assert_eq!(validate_as::<UiSettings>(&with("tray_grace_ms", 0)), Ok(()));
    let end = i64::try_from(*TRAY_GRACE_MS.end()).unwrap();
    assert_eq!(
        validate_as::<UiSettings>(&with("tray_grace_ms", end)),
        Ok(())
    );
    assert!(invalid(&with("tray_grace_ms", end + 1)));
    assert!(unparsed(&with("tray_grace_ms", -1)));
}

#[test]
fn device_rebuilds_may_be_none_but_not_past_their_range() {
    assert_eq!(
        validate_as::<UiSettings>(&with("device_rebuilds", 0)),
        Ok(())
    );
    let end = i64::from(*DEVICE_REBUILDS.end());
    assert_eq!(
        validate_as::<UiSettings>(&with("device_rebuilds", end)),
        Ok(())
    );
    assert!(invalid(&with("device_rebuilds", end + 1)));
    assert!(unparsed(&with("device_rebuilds", -1)));
}

#[test]
fn the_device_retry_may_be_at_once_but_not_past_its_range() {
    assert_eq!(
        validate_as::<UiSettings>(&with("device_retry_ms", 0)),
        Ok(())
    );
    let end = i64::try_from(*DEVICE_RETRY_MS.end()).unwrap();
    assert_eq!(
        validate_as::<UiSettings>(&with("device_retry_ms", end)),
        Ok(())
    );
    assert!(invalid(&with("device_retry_ms", end + 1)));
    assert!(unparsed(&with("device_retry_ms", -1)));
}

/// Qt 6.11.2's own frame latency (`QRhiD3D11::maxFrameLatency`) and update timer (`QPlatformWindow::requestUpdate`).
const QT_LATENCY: u32 = 2;
const QT_IDLE_MS: u64 = 5;

#[test]
fn the_frame_timing_knobs_default_to_what_qt_does_alone() {
    let s = UiSettings::from_defaults().unwrap();
    assert_eq!(
        (s.frame_latency, s.update_idle_ms),
        (QT_LATENCY, QT_IDLE_MS)
    );
    assert!(s.vblank_updates && s.present_vsync);
}

#[test]
fn the_frame_timing_numbers_may_be_zero_but_not_past_their_range() {
    for (key, end) in [
        ("frame_latency", i64::from(*FRAME_LATENCY.end())),
        (
            "update_idle_ms",
            i64::try_from(*UPDATE_IDLE_MS.end()).unwrap(),
        ),
    ] {
        assert_eq!(validate_as::<UiSettings>(&with(key, 0)), Ok(()), "{key}");
        assert_eq!(validate_as::<UiSettings>(&with(key, end)), Ok(()), "{key}");
        assert!(invalid(&with(key, end + 1)), "{key}");
        assert!(unparsed(&with(key, -1)), "{key}");
    }
    assert!(unparsed(&with("vblank_updates", "no")));
    assert!(unparsed(&with("present_vsync", 0)));
}

#[test]
fn page_rows_default_to_the_adr_0008_page() {
    assert!(PAGE_ROWS.contains(&UiSettings::from_defaults().unwrap().page_rows));
}

#[test]
fn unknown_keys_and_words_do_not_parse() {
    assert!(unparsed(&with("colour", "red")));
    assert!(unparsed(&with("mode", "sepia")));
    assert!(unparsed(&with("backdrop", "glass")));
    assert!(unparsed(&with("follow_accent", "yes")));
}

#[test]
fn each_backdrop_names_the_port_kind() {
    assert_eq!(Backdrop::None.kind(), BackdropKind::None);
    assert_eq!(Backdrop::Mica.kind(), BackdropKind::Mica);
    assert_eq!(Backdrop::Acrylic.kind(), BackdropKind::Acrylic);
    let s: UiSettings = parse_as(&with("backdrop", "mica")).unwrap();
    assert_eq!(s.backdrop, Backdrop::Mica);
}

#[test]
fn the_look_choice_carries_the_look_settings() {
    let mut t = with("theme", "et66");
    t.insert("mode".into(), "dark".into());
    t.insert("follow_accent".into(), true.into());
    t.insert("min_contrast".into(), 7.0.into());
    let s: UiSettings = parse_as(&t).unwrap();
    let c = s.choice();
    assert_eq!((c.theme.as_str(), c.mode), ("et66", ModeChoice::Dark));
    assert!(c.follow_accent && !c.reduced_motion);
    assert!((c.min_contrast - 7.0).abs() < f32::EPSILON);
}
