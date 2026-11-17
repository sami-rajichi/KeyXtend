//! The values the actor's ports take from `[voice]`: the focused box's look, Copy's wait, and a key the panel writes.

use super::*;
use kx_module_api::ports::{ClipWait, FieldConfig};

#[test]
fn the_ports_waits_hold_their_defaults_and_ranges() {
    let s = VoiceSettings::from_defaults().unwrap();
    assert_eq!((s.read_ms, s.copy_ms, s.copy_poll_ms), (150, 250, 10));
    assert_eq!(
        (READ_MS, COPY_MS, COPY_POLL_MS),
        (50..=1_000, 10..=2_000, 1..=100)
    );
}

#[test]
fn copy_poll_ms_must_not_pass_copy_ms() {
    let mut t = with("copy_ms", 50);
    t.insert("copy_poll_ms".into(), 60.into());
    assert!(refused(&t, "copy_poll_ms"));
}

#[test]
fn the_field_watch_reads_the_kind_alone_and_copy_waits_as_set() {
    let s = VoiceSettings::from_defaults().unwrap();
    let look = FieldConfig {
        read_ms: 150,
        read_chars: 0,
    };
    assert_eq!(s.field_config(), look);
    let wait = ClipWait {
        open_ms: 250,
        poll_ms: 10,
    };
    assert_eq!(s.copy_wait(), wait);
}

#[test]
fn a_key_the_panel_writes_gives_the_settings_after_it() {
    let s = VoiceSettings::from_defaults().unwrap();
    let moved = s.with(DEVICE, "cpu".into()).unwrap();
    assert_eq!(moved.device(), DevicePick::Cpu);
    let dir = moved.with(MODELS_DIR, r"D:\m".into()).unwrap();
    assert_eq!(
        (dir.models_dir.as_str(), dir.device()),
        (r"D:\m", DevicePick::Cpu)
    );
    assert!(
        s.with(DEVICE, "gpu!".into()).is_none(),
        "a refused value changes nothing"
    );
    assert!(
        s.with("engine", "x".into()).is_none(),
        "an unknown key changes nothing"
    );
}
