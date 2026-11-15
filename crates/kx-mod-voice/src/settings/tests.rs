//! The `[voice]` settings: the plan's defaults, every range's ends (the ports' own where they have one), the device,
//! place, folder and suffix rules, a bad pick dropped alone, and the values the parts take.

use super::*;
use kx_module_api::parse_as;
use kx_module_api::ports::{BarPlace, CardId, FetchAsk, Hundredths, JobLimits, SpeechAsk};
use std::time::Duration;

mod picks;
mod wiring;

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
    matches!(validate_as::<VoiceSettings>(t), Err(SettingsError::Invalid(why)) if why.contains(key))
}

/// True when the table does not even read as the settings.
fn unread(t: &toml::Table) -> bool {
    matches!(
        validate_as::<VoiceSettings>(t),
        Err(SettingsError::Parse(_))
    )
}

/// True when the table is taken.
fn taken(t: &toml::Table) -> bool {
    validate_as::<VoiceSettings>(t).is_ok()
}

/// The ends of `range`, widened for the table.
fn ends<T: Copy + Into<i64>>(range: &RangeInclusive<T>) -> (i64, i64) {
    ((*range.start()).into(), (*range.end()).into())
}

#[test]
fn the_defaults_validate_and_hold_the_plans_values() {
    assert!(SPEC.is_consistent());
    let s = VoiceSettings::from_defaults().unwrap();
    assert!(s.on && s.sounds && s.history && !s.silence_stop && !s.keep_voice);
    assert!(s.models_dir.is_empty() && s.picks.is_empty());
    assert_eq!((s.device.as_str(), s.place.as_str()), ("auto", "above"));
    assert_eq!((s.threads, s.idle_min, s.record_max_s), (0, 5, 600));
    assert_eq!(
        (s.piece_s, s.pad_ms, s.nothing_s, s.silence_stop_s),
        (30, 300, 5, 3)
    );
    assert_eq!(
        (s.debounce_ms, s.type_suffix.as_str(), s.type_gap_ms),
        (400, " ", 10)
    );
    assert_eq!((s.done_ms, s.wave_ms, s.level_hz), (9000, 300, 30));
    assert_eq!((s.history_max, s.voice_max_mb), (50, 1024));
    assert_eq!((s.card_margin_mb, s.card_diff), (512, 1));
    assert!((s.card_gain - 1.5).abs() < f64::EPSILON);
    assert_eq!((s.min_free_mb, s.job_mb), (2048, 512));
    assert_eq!(
        (s.ask_ms, s.load_s, s.warm_s, s.piece_wait_s),
        (3000, 60, 180, 120)
    );
    assert_eq!(
        (s.stall_s, s.restart_ms, s.restart_max_ms),
        (30, 1000, 30_000)
    );
}

#[test]
fn every_range_takes_its_ends_and_refuses_past_them() {
    let bounds = [
        ("threads", ends(&SpeechAsk::THREADS)),
        ("idle_min", ends(&JobLimits::IDLE_MIN)),
        ("record_max_s", ends(&SpeechAsk::RECORD_MAX_S)),
        ("piece_s", ends(&SpeechAsk::PIECE_S)),
        ("pad_ms", ends(&SpeechAsk::PAD_MS)),
        ("nothing_s", ends(&NOTHING_S)),
        ("silence_stop_s", ends(&SpeechAsk::SILENCE_STOP_S)),
        ("debounce_ms", ends(&DEBOUNCE_MS)),
        ("type_gap_ms", ends(&TYPE_GAP_MS)),
        ("done_ms", ends(&DONE_MS)),
        ("wave_ms", ends(&WAVE_MS)),
        ("level_hz", ends(&SpeechAsk::LEVEL_HZ)),
        ("history_max", ends(&HISTORY_MAX)),
        ("voice_max_mb", ends(&VOICE_MAX_MB)),
        ("card_margin_mb", ends(&CardUse::MARGIN_MB)),
        ("card_diff", ends(&CARD_DIFF)),
        ("min_free_mb", ends(&FetchAsk::MIN_FREE_MB)),
        ("job_mb", ends(&JobLimits::JOB_MB)),
        ("ask_ms", ends(&JobLimits::ASK_MS)),
        ("load_s", ends(&JobLimits::LOAD_S)),
        ("warm_s", ends(&JobLimits::WARM_S)),
        ("piece_wait_s", ends(&JobLimits::PIECE_WAIT_S)),
        ("stall_s", ends(&FetchAsk::STALL_S)),
        ("restart_ms", ends(&JobLimits::RESTART_MS)),
        ("restart_max_ms", ends(&JobLimits::RESTART_MAX_MS)),
        ("read_ms", ends(&READ_MS)),
        ("copy_ms", ends(&COPY_MS)),
        ("copy_poll_ms", ends(&COPY_POLL_MS)),
    ];
    for (key, (lo, hi)) in bounds {
        let mut top = with(key, hi);
        if key == "restart_ms" {
            top.insert("restart_max_ms".into(), hi.into());
        }
        assert!(taken(&with(key, lo)) && taken(&top), "{key}");
        let (below, above) = (with(key, lo - 1), with(key, hi + 1));
        assert!(refused(&below, key) || unread(&below), "{key} = {}", lo - 1);
        assert!(refused(&above, key) || unread(&above), "{key} = {}", hi + 1);
    }
}

#[test]
fn the_plans_own_ranges_hold() {
    assert_eq!(
        (NOTHING_S, DEBOUNCE_MS, TYPE_GAP_MS),
        (2..=60, 100..=2000, 0..=200)
    );
    assert_eq!(
        (DONE_MS, WAVE_MS, HISTORY_MAX),
        (2000..=60_000, 0..=2000, 1..=500)
    );
    assert_eq!(
        (VOICE_MAX_MB, CARD_DIFF, TYPE_SUFFIX_CHARS),
        (64..=65_536, 0..=5, 8)
    );
    assert_eq!(CARD_GAIN, 1.0..=10.0);
}

#[test]
fn card_gain_takes_its_ends_and_refuses_past_them_or_nan() {
    for good in [1.0, 2.25, 10.0] {
        assert!(taken(&with("card_gain", good)), "{good}");
    }
    for bad in [0.99, 10.01, f64::NAN, -1.0] {
        assert!(refused(&with("card_gain", bad), "card_gain"), "{bad}");
    }
}

#[test]
fn restart_ms_must_not_pass_restart_max_ms() {
    let mut t = with("restart_ms", 5000);
    t.insert("restart_max_ms".into(), 4000.into());
    assert!(refused(&t, "restart_ms"));
}

#[test]
fn device_is_auto_cpu_or_a_cards_id() {
    for good in ["auto", "cpu", "Vulkan0", "Vulkan1", "Metal.0"] {
        assert!(taken(&with("device", good)), "{good}");
    }
    let long = "v".repeat(*kx_module_api::ports::TAG_LEN.end() + 1);
    for bad in ["", "Vulkan 0", "gpu!", "بطاقة", long.as_str()] {
        assert!(refused(&with("device", bad), "device"), "{bad}");
    }
}

#[test]
fn place_is_one_of_its_three_names() {
    for (name, place) in PLACES {
        let s = parse_as::<VoiceSettings>(&with("place", name)).unwrap();
        assert_eq!(s.bar_place(), place, "{name}");
        assert_eq!(place_name(place), name);
    }
    for bad in ["left", "", "Above", "فوق"] {
        assert!(refused(&with("place", bad), "place"), "{bad}");
    }
    assert!(unread(&with("place", 2)));
}

#[test]
fn models_dir_is_empty_or_a_plain_drive_path() {
    for good in ["", r"D:\KeyXtend\models", r"C:\Users\مستخدم\models"] {
        assert!(taken(&with("models_dir", good)), "{good}");
    }
    for bad in [
        "models",
        r"\\server\share\models",
        r"D:\a\..\b",
        "C:foo",
        r"D:\NUL\x",
    ] {
        assert!(refused(&with("models_dir", bad), "models_dir"), "{bad}");
    }
}

#[test]
fn type_suffix_is_short_and_shows() {
    let eight = "·".repeat(TYPE_SUFFIX_CHARS);
    for good in ["", " ", ". ", "، ", eight.as_str()] {
        assert!(taken(&with("type_suffix", good)), "{good:?}");
    }
    let nine = "·".repeat(TYPE_SUFFIX_CHARS + 1);
    for bad in [nine.as_str(), "\n", "a\tb", "\u{200B}"] {
        assert!(refused(&with("type_suffix", bad), "type_suffix"), "{bad:?}");
    }
}

#[test]
fn an_unknown_key_is_refused() {
    assert!(unread(&with("engine", "whisper")));
}

#[test]
fn the_job_and_the_ask_get_the_settings_in_their_units() {
    let s = VoiceSettings::from_defaults().unwrap();
    let job = s.job();
    let (load, wait) = ((job.ask_ms, job.load_s, job.warm_s), job.piece_wait_s);
    assert_eq!(
        (job.job_mb, job.idle_min, load, wait),
        (512, 5, (3000, 60, 180), 120)
    );
    assert_eq!((job.restart_ms, job.restart_max_ms), (1000, 30_000));
    assert_eq!(job.check(), Ok(()));
    let ask = s.ask(None, None);
    assert_eq!(
        ask.check(),
        Ok(()),
        "a listen-only ask from the defaults passes"
    );
    let sizes = (ask.threads, ask.piece_s, ask.pad_ms, ask.level_hz);
    assert_eq!(sizes, (0, 30, 300, 30));
    assert_eq!((ask.record_max_s, ask.silence_stop_s), (600, None));
    let mut on = s;
    on.silence_stop = true;
    assert_eq!(on.ask(None, None).silence_stop_s, Some(3));
}

#[test]
fn the_device_place_rules_and_times_come_in_their_types() {
    let s = VoiceSettings::from_defaults().unwrap();
    assert_eq!(
        (s.device(), s.bar_place()),
        (DevicePick::Auto, BarPlace::Above)
    );
    let mut on = s.clone();
    on.device = "Vulkan0".into();
    assert_eq!(
        on.device(),
        DevicePick::Card(CardId::new("Vulkan0").unwrap())
    );
    let rules = s.card_rules();
    assert_eq!((rules.gain, rules.diff), (Hundredths(150), 1));
    let (ms, secs) = (Duration::from_millis, Duration::from_secs);
    assert_eq!((s.debounce(), s.nothing()), (ms(400), secs(5)));
    assert_eq!((s.done(), s.wave()), (secs(9), ms(300)));
}
