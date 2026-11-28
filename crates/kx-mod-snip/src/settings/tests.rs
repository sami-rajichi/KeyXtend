//! The `[snip]` settings: the plan's defaults, every range's ends and any value near them, the frame rates, the
//! keyboard's three ways, the folder kept as typed, the write-back, the cap, and Copy's wait; the editor's place is in
//! `editor`.

mod editor;

use super::*;
use std::time::Duration;

use kx_module_api::parse_as;
use kx_module_api::ports::{ClipWait, PickLimits, RecordAsk, RecordWaits, SHOT_CAP_MB};
use proptest::prelude::*;

/// The defaults as a table.
fn defaults() -> toml::Table {
    SPEC.defaults.parse().unwrap()
}

/// The defaults with `key` set to `value`.
fn with(key: &str, value: impl Into<toml::Value>) -> toml::Table {
    let mut t = defaults();
    t.insert(key.to_owned(), value.into());
    t
}

/// True when the table is refused by a rule that names `key`.
fn refused(t: &toml::Table, key: &str) -> bool {
    matches!(validate_as::<SnipSettings>(t), Err(SettingsError::Invalid(why)) if why.contains(key))
}

/// True when the table does not even read as the settings.
fn unread(t: &toml::Table) -> bool {
    matches!(validate_as::<SnipSettings>(t), Err(SettingsError::Parse(_)))
}

/// True when the table is taken.
fn taken(t: &toml::Table) -> bool {
    validate_as::<SnipSettings>(t).is_ok()
}

/// The table is read as the settings.
fn read(t: &toml::Table) -> SnipSettings {
    parse_as(t).unwrap()
}

/// The ends of `range`, widened for the table.
fn ends<T: Copy + Into<i64>>(range: &RangeInclusive<T>) -> (i64, i64) {
    ((*range.start()).into(), (*range.end()).into())
}

/// Each ranged key with its ends.
fn ranges() -> [(&'static str, (i64, i64)); 33] {
    [
        ("aside_ms", ends(&ASIDE_MS)),
        ("frame_min_px", ends(&FRAME_MIN_PX)),
        ("shown_ms", ends(&SHOWN_MS)),
        ("countdown_step_s", ends(&COUNTDOWN_STEP_S)),
        ("max_min_step", ends(&MAX_MIN_STEP)),
        ("encode_ms", ends(&ENCODE_MS)),
        ("encode_mb_ms", ends(&ENCODE_MB_MS)),
        ("encode_idle_s", ends(&ENCODE_IDLE_S)),
        ("encode_restart_ms", ends(&ENCODE_RESTART_MS)),
        ("countdown_s", ends(&COUNTDOWN_S)),
        ("max_min", ends(&RecordAsk::MAX_MIN)),
        ("copied_days", ends(&COPIED_DAYS)),
        ("min_free_mb", ends(&RecordAsk::MIN_FREE_MB)),
        ("undo_max", ends(&UNDO_MAX)),
        ("stroke_max", ends(&STROKE_MAX)),
        ("shot_cap_mb", ends(&SHOT_CAP_MB)),
        ("record_mb", ends(&RecordAsk::JOB_MB)),
        ("copy_ms", ends(&COPY_MS)),
        ("copy_poll_ms", ends(&COPY_POLL_MS)),
        ("copy_again", ends(&COPY_AGAIN)),
        ("copy_again_ms", ends(&COPY_AGAIN_MS)),
        ("note_s", ends(&NOTE_S)),
        ("record_start_ms", ends(&RecordWaits::START_MS)),
        ("record_tick_ms", ends(&RecordWaits::TICK_MS)),
        ("record_end_ms", ends(&RecordWaits::END_MS)),
        ("switch_ms", ends(&RecordWaits::SWITCH_MS)),
        ("speed_ms", ends(&RecordWaits::SPEED_MS)),
        ("speed_mb_ms", ends(&RecordWaits::SPEED_MB_MS)),
        ("pick_mb", ends(&PickLimits::JOB_MB)),
        ("dialog_min", ends(&PickLimits::DIALOG_MIN)),
        ("pick_idle_s", ends(&PickLimits::IDLE_S)),
        ("pick_restart_ms", ends(&PickLimits::RESTART_MS)),
        ("pick_restart_max_ms", ends(&PickLimits::RESTART_MAX_MS)),
    ]
}

/// `rect` as the editor's table.
fn place(rect: Rect) -> toml::Table {
    toml::Table::try_from(rect).unwrap()
}

/// An editor somewhere on the screens.
const PLACED: Rect = Rect {
    x: -8,
    y: 40,
    w: 900,
    h: 600,
};

#[test]
fn the_defaults_validate_and_hold_the_plans_values() {
    assert!(SPEC.is_consistent());
    let s = SnipSettings::from_defaults().unwrap();
    assert_eq!(s.keyboard, SnipKeyboard::Bubble);
    assert!(s.save_dir.is_empty() && s.editor.is_none());
    assert_eq!((s.countdown_s, s.fps, s.max_min), (3, 30, 30));
    assert!(!s.mic_on && !s.sound_on);
    assert_eq!((s.copied_days, s.min_free_mb, s.undo_max), (1, 2048, 50));
    assert_eq!((s.shot_cap_mb, s.record_mb), (256, 512));
    assert_eq!((s.copy_ms, s.copy_poll_ms, s.copy_again_ms), (250, 10, 400));
    assert_eq!((s.copy_again, s.note_s, s.aside_ms), (2, 4, 500));
    assert_eq!(s.frame_min_px, 8, "plan 2026-11-26");
    let wait = ClipWait {
        open_ms: 250,
        poll_ms: 10,
    };
    assert_eq!(s.copy_wait(), wait);
}

// The shared ranges are the ports' own, defined once in `kx-ipc`; only the module's own are pinned to the plan here.
#[test]
fn the_modules_own_ranges_are_the_plans() {
    let own = [ends(&COUNTDOWN_S), ends(&COPIED_DAYS), ends(&UNDO_MAX)];
    assert_eq!(own, [(0, 10), (1, 7), (10, 200)]);
    let copy = [ends(&COPY_MS), ends(&COPY_POLL_MS), ends(&COPY_AGAIN_MS)];
    assert_eq!(copy, [(10, 2_000), (1, 100), (100, 5_000)]);
    assert_eq!([ends(&COPY_AGAIN), ends(&NOTE_S)], [(0, 5), (2, 30)]);
    assert_eq!(ends(&ASIDE_MS), (100, 2_000));
    assert_eq!(ends(&FRAME_MIN_PX), (1, 64));
}

#[test]
fn the_copys_poll_never_passes_its_wait() {
    let mut t = with("copy_ms", 20);
    t.insert("copy_poll_ms".into(), 21.into());
    assert!(refused(&t, "copy_poll_ms"));
    t.insert("copy_poll_ms".into(), 20.into());
    assert!(taken(&t));
}

#[test]
fn the_dialogs_first_restart_wait_never_passes_the_longest() {
    let longest = *PickLimits::RESTART_MAX_MS.start();
    let mut t = with("pick_restart_max_ms", longest);
    t.insert("pick_restart_ms".into(), (longest + 1).into());
    assert!(refused(&t, "pick_restart_ms"));
    t.insert("pick_restart_ms".into(), longest.into());
    assert!(taken(&t));
}

#[test]
fn the_jobs_waits_the_tick_and_the_steps_carry_their_keys() {
    let s = SnipSettings::from_defaults().unwrap();
    let waits = RecordWaits {
        start_ms: s.record_start_ms,
        tick_ms: s.record_tick_ms,
        end_ms: s.record_end_ms,
        switch_ms: s.switch_ms,
        speed_ms: s.speed_ms,
        speed_mb_ms: s.speed_mb_ms,
    };
    assert_eq!(s.record_waits(), waits);
    let limits = PickLimits {
        job_mb: s.pick_mb,
        dialog_min: s.dialog_min,
        idle_s: s.pick_idle_s,
        restart_ms: s.pick_restart_ms,
        restart_max_ms: s.pick_restart_max_ms,
    };
    assert_eq!(s.pick_limits(), limits);
    assert_eq!(limits.check(), Ok(()), "the port's own check");
    let e = s.encode_waits();
    let encode = (
        s.encode_ms,
        s.encode_mb_ms,
        s.encode_idle_s,
        s.encode_restart_ms,
    );
    assert_eq!((e.ms, e.mb_ms, e.idle_s, e.restart_ms), encode);
    assert_eq!(s.shown(), Duration::from_millis(u64::from(s.shown_ms)));
    let steps = (COUNTDOWN_STEP_S.end(), MAX_MIN_STEP.end());
    assert!(steps.0 <= COUNTDOWN_S.end() && steps.1 <= RecordAsk::MAX_MIN.end());
}

#[test]
fn every_range_takes_its_ends_and_refuses_past_them() {
    for (key, (lo, hi)) in ranges() {
        assert!(taken(&with(key, lo)) && taken(&with(key, hi)), "{key}");
        let (below, above) = (with(key, lo - 1), with(key, hi + 1));
        // Below 0 an unsigned value does not read; any other value past an end is refused by its own rule.
        let below_out = if lo == 0 {
            unread(&below)
        } else {
            refused(&below, key)
        };
        assert!(below_out, "{key} = {}", lo - 1);
        assert!(refused(&above, key), "{key} = {}", hi + 1);
    }
}

#[test]
fn fps_takes_the_recorders_rates_alone() {
    for fps in RecordAsk::FPS {
        assert_eq!(read(&with("fps", i64::from(fps))).fps, fps);
    }
    for fps in [0, 14, 16, 29, 31, 60] {
        assert!(refused(&with("fps", fps), "fps"), "{fps}");
    }
    assert!(unread(&with("fps", -1)) && unread(&with("fps", 256)));
}

#[test]
fn keyboard_takes_its_three_names_and_no_other() {
    let ways = [
        ("bubble", SnipKeyboard::Bubble),
        ("hidden", SnipKeyboard::Hidden),
        ("shown", SnipKeyboard::Shown),
    ];
    for (name, way) in ways {
        assert_eq!(read(&with("keyboard", name)).keyboard, way, "{name}");
    }
    for bad in ["Bubble", "off", "", "visible"] {
        assert!(unread(&with("keyboard", bad)), "{bad}");
    }
    assert!(unread(&with("keyboard", 1)));
}

#[test]
fn save_dir_is_kept_as_typed() {
    for dir in ["", r"D:\Snips", "no such folder"] {
        assert_eq!(read(&with("save_dir", dir)).save_dir, dir);
    }
}

#[test]
fn the_settings_write_back_as_a_section_that_reads_the_same() {
    let mut s = SnipSettings::from_defaults().unwrap();
    let back = toml::Table::try_from(&s).unwrap();
    assert_eq!(back, defaults(), "an unplaced editor writes back empty");
    s.editor = Some(PLACED);
    s.keyboard = SnipKeyboard::Shown;
    assert_eq!(read(&toml::Table::try_from(&s).unwrap()), s);
}

#[test]
fn an_unknown_key_does_not_read() {
    assert!(unread(&with("ocr", true)));
}

#[test]
fn the_freezes_cap_is_the_checked_type_of_shot_cap_mb() {
    let s = read(&with("shot_cap_mb", 1_000));
    assert_eq!(s.shot_cap().map(ShotCap::mb), Some(1_000));
    let shot_cap_mb = SHOT_CAP_MB.start() - 1;
    assert_eq!(SnipSettings { shot_cap_mb, ..s }.shot_cap(), None);
}

/// Any ranged key with its ends, and a value from one below its range to one above.
fn ranged() -> impl Strategy<Value = (&'static str, (i64, i64), i64)> {
    (0..ranges().len()).prop_flat_map(|pick| {
        let (key, (lo, hi)) = ranges()[pick];
        (Just(key), Just((lo, hi)), lo - 1..=hi + 1)
    })
}

proptest! {
    #[test]
    fn a_ranged_value_is_taken_exactly_within_its_range((key, (lo, hi), value) in ranged()) {
        prop_assert_eq!(taken(&with(key, value)), (lo..=hi).contains(&value));
    }

    #[test]
    fn a_frame_rate_is_taken_exactly_when_the_recorder_lists_it(fps in any::<u8>()) {
        prop_assert_eq!(taken(&with("fps", i64::from(fps))), RecordAsk::FPS.contains(&fps));
    }
}
