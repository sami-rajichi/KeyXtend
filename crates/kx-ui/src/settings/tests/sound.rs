//! The `[ui]` sound settings: every sound on at the owner's soft volume, and the volume and the idle time within their
//! ranges.

use super::*;
use kx_theme::sounds::Sounds;

/// About how long a sound device takes to play out what it was handed, in ms.
const DEVICE_LATENCY_MS: f64 = 100.0;

/// The first and last values of `range` as TOML integers.
fn ends<T: Copy + Into<i64>>(range: &RangeInclusive<T>) -> (i64, i64) {
    ((*range.start()).into(), (*range.end()).into())
}

/// The four sound switches: key, right-click, pick-up and drop.
fn switches(s: &UiSettings) -> [bool; 4] {
    [s.sound_key, s.sound_rclick, s.sound_pick, s.sound_drop]
}

/// The volume the owner picked by ear on 2026-10-15, as 50, 35 and 25 were too loud.
const SOFT: u8 = 15;

#[test]
fn every_sound_starts_on_at_a_soft_volume_and_each_turns_off_alone() {
    let s = UiSettings::from_defaults().unwrap();
    assert_eq!(switches(&s), [true; 4]);
    assert_eq!((s.sound_volume, s.sound_idle_ms), (SOFT, 3_000));
    let keys = ["sound_key", "sound_rclick", "sound_pick", "sound_drop"];
    for (i, key) in keys.into_iter().enumerate() {
        let off: UiSettings = parse_as(&with(key, false)).unwrap();
        let mut want = [true; 4];
        want[i] = false;
        assert_eq!(switches(&off), want, "{key}");
        assert!(unparsed(&with(key, "on")), "{key}");
    }
}

#[test]
fn the_volume_and_the_idle_time_stay_in_their_range() {
    let idle = ends(&SOUND_IDLE_MS);
    for (key, (start, end)) in [
        ("sound_volume", ends(&SOUND_VOLUME)),
        ("sound_idle_ms", idle),
    ] {
        for edge in [start, end] {
            assert_eq!(validate_as::<UiSettings>(&with(key, edge)), Ok(()), "{key}");
        }
        assert!(invalid(&with(key, end + 1)), "{key}");
        assert!(unparsed(&with(key, -1)), "{key}");
    }
    assert!(
        invalid(&with("sound_idle_ms", idle.0 - 1)),
        "under the floor"
    );
}

#[test]
fn the_idle_floor_is_twice_the_longest_sound_and_the_device() {
    let longest = Sounds::shipped().unwrap().format.max_ms;
    let floor = f64::from(*SOUND_IDLE_MS.start());
    assert!(floor >= 2.0 * (longest + DEVICE_LATENCY_MS), "{floor} ms");
}
